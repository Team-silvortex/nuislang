---

# Nuis Intermediate Representation

## Draft Specification v0.01

---

# 1. Overview

Nuis is a semantics-first execution architecture for heterogeneous computing systems.

This document describes the IR boundary of the `nuis` toolchain itself, with
`AOT-first` as the default and governing profile. Optional runtime-facing
integration is out of scope here unless it preserves the same IR boundary.

The current architecture separates program semantics into three layers:

```
NIR        — Semantic intent
YIR        — Execution topology
Fabric IR  — Data propagation semantics
```

NIR captures stable program intent.

YIR captures execution topology and semantic ordering.

Fabric IR captures data movement, visibility, and synchronization surfaces.

```
Program =
SemanticIntent (NIR)
-> ExecutionGraph (YIR)
+ DataFabricGraph (Fabric IR)
```

This separation keeps semantic intent, execution order, and data propagation independently analyzable.

---

# 2. Core Principles

### 2.1 Orthogonality

Execution and data movement are modeled independently.

```
compute ≠ data movement
```

NIR describes **what the program means**.

YIR describes **how execution is ordered**.

Fabric IR describes **how data moves and becomes visible across domains**.

---

### 2.2 Static Graph Model

All graphs are statically compiled.

```
No runtime graph scheduling
No dynamic topology
```

The entire execution and data fabric topology must be known at compile time.

---

### 2.3 Minimal Primitive Set

Fabric primitives are fixed.

Extensions must **compose primitives** rather than introduce new ones.

This guarantees verifier tractability.

---

### 2.4 Immutable-First Data Model

Persistent data is immutable.

Mutability is only allowed within explicitly bounded transient stages.

---

### 2.5 Hardware Independence

The IR is not specialized for any specific hardware.

Current implementations may lower to CPU cores, but the model is designed to support future dedicated fabric hardware while keeping the AOT semantic frame stable.

---

# 3. Execution IR (YIR)

YIR represents execution topology.

It describes computation, synchronization, and resource usage.

Current implementation priority in this repository:

* hand-authored YIR source
* static verification
* reference execution
* explicit graph edges instead of text-order scheduling
* mod-registered instruction sets (`nustar`-style expansion point)
* Fabric behavior represented first as explicit YIR-side data actions

### Core Operations

```
compute
move
sync
effect
resource
```

Minimal handwritten prototype directives:

```text
yir <version>
resource <name> <kind>
<mod>.<instr> <name> <resource> [args...]
edge <dep|effect|lifetime|xfer> <from> <to>
```

Preferred handwritten style in this repository:

* use explicit resource kinds such as `cpu.arm64`, `shader.render`,
  `kernel.apple`, and `data.fabric`
* keep `data.*` nodes attached to `data.fabric` resources instead of reusing
  compute resources
* prefer typed scalar constructors when the scalar type is known, such as
  `cpu.const_i64`, `shader.const_i64`, or `kernel.const_f32`
* prefer canonical ops such as `shader.sample` / `shader.sample_uv` over
  compatibility aliases in new examples

Current built-in / registered examples:

```text
cpu.const <name> <resource> <value>
cpu.null <name> <resource>
cpu.borrow <name> <resource> <ptr>
cpu.move_ptr <name> <resource> <ptr>
cpu.neg <name> <resource> <input>
cpu.not <name> <resource> <input>
cpu.add <name> <resource> <lhs> <rhs>
cpu.add_i32 <name> <resource> <lhs> <rhs>
cpu.add_f32 <name> <resource> <lhs> <rhs>
cpu.add_f64 <name> <resource> <lhs> <rhs>
cpu.sub <name> <resource> <lhs> <rhs>
cpu.sub_i32 <name> <resource> <lhs> <rhs>
cpu.sub_f32 <name> <resource> <lhs> <rhs>
cpu.sub_f64 <name> <resource> <lhs> <rhs>
cpu.mul <name> <resource> <lhs> <rhs>
cpu.mul_i32 <name> <resource> <lhs> <rhs>
cpu.mul_f32 <name> <resource> <lhs> <rhs>
cpu.mul_f64 <name> <resource> <lhs> <rhs>
cpu.div <name> <resource> <lhs> <rhs>
cpu.div_i32 <name> <resource> <lhs> <rhs>
cpu.div_f32 <name> <resource> <lhs> <rhs>
cpu.div_f64 <name> <resource> <lhs> <rhs>
cpu.rem <name> <resource> <lhs> <rhs>
cpu.eq <name> <resource> <lhs> <rhs>
cpu.eq_i32 <name> <resource> <lhs> <rhs>
cpu.eq_f32 <name> <resource> <lhs> <rhs>
cpu.eq_f64 <name> <resource> <lhs> <rhs>
cpu.ne <name> <resource> <lhs> <rhs>
cpu.lt <name> <resource> <lhs> <rhs>
cpu.lt_i32 <name> <resource> <lhs> <rhs>
cpu.lt_f32 <name> <resource> <lhs> <rhs>
cpu.lt_f64 <name> <resource> <lhs> <rhs>
cpu.gt <name> <resource> <lhs> <rhs>
cpu.gt_i32 <name> <resource> <lhs> <rhs>
cpu.gt_f32 <name> <resource> <lhs> <rhs>
cpu.gt_f64 <name> <resource> <lhs> <rhs>
cpu.le <name> <resource> <lhs> <rhs>
cpu.ge <name> <resource> <lhs> <rhs>
cpu.and <name> <resource> <lhs> <rhs>
cpu.or <name> <resource> <lhs> <rhs>
cpu.xor <name> <resource> <lhs> <rhs>
cpu.shl <name> <resource> <lhs> <rhs>
cpu.shr <name> <resource> <lhs> <rhs>
cpu.madd <name> <resource> <lhs> <rhs> <acc>
cpu.select <name> <resource> <cond> <then> <else>
cpu.alloc_node <name> <resource> <value> <next_ptr>
cpu.alloc_buffer <name> <resource> <len> <fill>
cpu.load_value <name> <resource> <ptr>
cpu.load_next <name> <resource> <ptr>
cpu.buffer_len <name> <resource> <buffer_ptr>
cpu.load_at <name> <resource> <buffer_ptr> <index>
cpu.store_value <name> <resource> <ptr> <value>
cpu.store_next <name> <resource> <ptr> <next_ptr>
cpu.store_at <name> <resource> <buffer_ptr> <index> <value>
cpu.is_null <name> <resource> <ptr>
cpu.free <name> <resource> <ptr>
cpu.const_bool <name> <resource> <true|false>
cpu.const_i32 <name> <resource> <value>
cpu.const_i64 <name> <resource> <value>
cpu.const_f32 <name> <resource> <value>
cpu.const_f64 <name> <resource> <value>
cpu.struct <name> <resource> <type_name> <field=value>...
cpu.field <name> <resource> <struct> <field_name>
cpu.cast_i32_to_i64 <name> <resource> <input>
cpu.cast_i64_to_i32 <name> <resource> <input>
cpu.cast_i32_to_f32 <name> <resource> <input>
cpu.cast_i32_to_f64 <name> <resource> <input>
cpu.cast_f32_to_f64 <name> <resource> <input>
cpu.cast_f64_to_f32 <name> <resource> <input>
cpu.target_config <name> <resource> <arch> <abi> <vector_bits>
cpu.bind_core <name> <resource> <core_index>
cpu.window <name> <resource> <width> <height> <title>
cpu.input_i64 <name> <resource> <channel> <default> [<min> <max> <step>]
cpu.tick_i64 <name> <resource> <start> <step>
cpu.present_frame <name> <resource> <frame>
cpu.print <name> <resource> <input>

Current handwritten `YIR` now includes a first typed value surface:

* `bool`
* `i32`
* `i64`
* `f32`
* `f64`
* named `struct`

`cpu.struct` and `cpu.field` currently focus on value expression and field
extraction. The LLVM path already supports typed constants and struct field
access; a full stable struct aggregate ABI is still provisional.

The current CPU surface also has a first typed arithmetic slice:

* `add_i32 / sub_i32 / mul_i32 / div_i32`
* `add_f32 / sub_f32 / mul_f32 / div_f32`
* `add_f64 / sub_f64 / mul_f64 / div_f64`

It now also has a first typed comparison/conversion slice:

* `eq_i32 / lt_i32 / gt_i32`
* `eq_f32 / lt_f32 / gt_f32`
* `eq_f64 / lt_f64 / gt_f64`
* `cast_i32_to_i64 / cast_i64_to_i32`
* `cast_i32_to_f32 / cast_i32_to_f64`
* `cast_f32_to_f64 / cast_f64_to_f32`
kernel.target_config <name> <resource> <arch> <runtime> <lane_width>
kernel.const_bool <name> <resource> <value>
kernel.const_i32 <name> <resource> <value>
kernel.const_i64 <name> <resource> <value>
kernel.const_f32 <name> <resource> <value>
kernel.const_f64 <name> <resource> <value>
kernel.tensor <name> <resource> <rows> <cols> <csv-elements>
kernel.fill <name> <resource> <rows> <cols> <value>
kernel.splat <name> <resource> <rows> <cols> <scalar>
kernel.add <name> <resource> <lhs> <rhs>
kernel.mul <name> <resource> <lhs> <rhs>
kernel.add_scalar <name> <resource> <tensor> <scalar>
kernel.mul_scalar <name> <resource> <tensor> <scalar>
kernel.add_i32 <name> <resource> <lhs> <rhs>
kernel.mul_i32 <name> <resource> <lhs> <rhs>
kernel.add_f32 <name> <resource> <lhs> <rhs>
kernel.mul_f32 <name> <resource> <lhs> <rhs>
kernel.add_f64 <name> <resource> <lhs> <rhs>
kernel.mul_f64 <name> <resource> <lhs> <rhs>
kernel.matmul <name> <resource> <lhs> <rhs>
kernel.add_bias <name> <resource> <input> <bias>
kernel.shape <name> <resource> <input>
kernel.rows <name> <resource> <input>
kernel.cols <name> <resource> <input>
kernel.row <name> <resource> <input>
kernel.col <name> <resource> <input>
kernel.element_at <name> <resource> <input> <row> <col>
kernel.reshape <name> <resource> <input> <rows> <cols>
kernel.slice <name> <resource> <input> <row_offset> <col_offset> <rows> <cols>
kernel.broadcast <name> <resource> <input> <rows> <cols>
kernel.transpose <name> <resource> <input>
kernel.reduce_sum <name> <resource> <input>
kernel.reduce_sum_axis <name> <resource> <input> <rows|cols>
kernel.reduce_max <name> <resource> <input>
kernel.reduce_max_axis <name> <resource> <input> <rows|cols>
kernel.reduce_mean <name> <resource> <input>
kernel.reduce_mean_axis <name> <resource> <input> <rows|cols>
kernel.reduce_min <name> <resource> <input>
kernel.reduce_min_axis <name> <resource> <input> <rows|cols>
kernel.argmax <name> <resource> <input>
kernel.argmax_axis <name> <resource> <input> <rows|cols>
kernel.argmin <name> <resource> <input>
kernel.argmin_axis <name> <resource> <input> <rows|cols>
kernel.sort <name> <resource> <input>
kernel.topk <name> <resource> <input> <k>
kernel.topk_axis <name> <resource> <input> <k> <rows|cols>
kernel.relu <name> <resource> <input>
kernel.print <name> <resource> <input>
data.move <name> <resource> <input> <to>
data.copy_window <name> <resource> <input> <offset> <len>
data.immutable_window <name> <resource> <input> <offset> <len>
data.marker <name> <resource> <tag>
data.bind_core <name> <resource> <core_index>
data.output_pipe <name> <resource> <input>
data.input_pipe <name> <resource> <pipe>
data.handle_table <name> <resource> <slot=resource> [slot=resource...]
shader.const <name> <resource> <value>
shader.const_bool <name> <resource> <value>
shader.const_i32 <name> <resource> <value>
shader.const_i64 <name> <resource> <value>
shader.const_f32 <name> <resource> <value>
shader.const_f64 <name> <resource> <value>
shader.add <name> <resource> <lhs> <rhs>
shader.sub <name> <resource> <lhs> <rhs>
shader.mul <name> <resource> <lhs> <rhs>
shader.add_i32 <name> <resource> <lhs> <rhs>
shader.mul_i32 <name> <resource> <lhs> <rhs>
shader.add_f32 <name> <resource> <lhs> <rhs>
shader.mul_f32 <name> <resource> <lhs> <rhs>
shader.add_f64 <name> <resource> <lhs> <rhs>
shader.mul_f64 <name> <resource> <lhs> <rhs>
shader.target <name> <resource> <format> <width> <height>
shader.viewport <name> <resource> <width> <height>
shader.pipeline <name> <resource> <shading_model> <topology>
shader.vertex_layout <name> <resource> <stride> <csv-attributes>
shader.vertex_buffer <name> <resource> <vertex_count> <csv-elements>
shader.index_buffer <name> <resource> <csv-indices>
shader.blend_state <name> <resource> <enabled> <mode>
shader.depth_state <name> <resource> <test_enabled> <write_enabled> <compare>
shader.raster_state <name> <resource> <cull_mode> <front_face>
shader.render_state <name> <resource> <pipeline> <blend> <depth> <raster>
shader.uv <name> <resource> <u_1024> <v_1024>
shader.texture2d <name> <resource> <format> <width> <height> <csv-texels>
shader.sampler <name> <resource> <filter> <address_mode>
shader.uniform <name> <resource> <slot> <value>
shader.storage <name> <resource> <slot> <value>
shader.attachment <name> <resource> <slot> <target>
shader.texture_binding <name> <resource> <slot> <texture>
shader.sampler_binding <name> <resource> <slot> <sampler>
shader.vertex_layout_binding <name> <resource> <slot> <vertex_layout>
shader.vertex_binding <name> <resource> <slot> <vertex_buffer>
shader.index_binding <name> <resource> <slot> <index_buffer>
shader.bind_set <name> <resource> <pipeline> <binding> [binding...]
shader.pack_ball_state <name> <resource> <color> <speed>
shader.begin_pass <name> <resource> <target> <pipeline> <viewport>
shader.clear <name> <resource> <target> <fill>
shader.overlay <name> <resource> <base> <top>
shader.sample <name> <resource> <texture> <sampler> <x> <y>
shader.sample_uv <name> <resource> <texture> <sampler> <uv>
shader.sample_nearest <name> <resource> <texture> <sampler> <x> <y>
shader.sample_uv_nearest <name> <resource> <texture> <sampler> <uv>
shader.sample_uv_linear <name> <resource> <texture> <sampler> <uv>
shader.dispatch <name> <resource> <input>
shader.draw_instanced <name> <resource> <pass> <packet> <vertex_count> <instance_count> [bind_set]
shader.draw_ball <name> <resource> <packet>
shader.draw_sphere <name> <resource> <packet>
shader.print <name> <resource> <input>
```

Important boundary note:

* `cpu.window`, `cpu.input_i64`, and `cpu.present_frame` are not YIR-core UI semantics.
* They are current `cpu`-mod extension ops used by the reference preview/runtime path.
* A different frontend, runtime adapter, or future framework can consume the same YIR graph without depending on these specific ops.
* `cpu.borrow`, `cpu.borrow_end`, and `cpu.move_ptr` are the first Rust-like ownership surface for the pure CPU domain: reads may flow through borrowed pointers, while writes and frees remain ownership-sensitive until the borrow scope ends.
* `cpu.alloc_node / alloc_buffer / load_* / store_* / free` are an early reference prototype for addressable objects and pointer-like semantics. They are intentionally narrow and currently model controlled heap-node and heap-buffer surfaces rather than a full general memory model.
* `lifetime` is now part of the current handwritten reference style for ownership-sensitive `res` flow. In the current prototype, `Own` and `Write` resource accesses require a `lifetime` edge in addition to ordinary `dep` or `xfer` ordering.
* The current CPU verifier treats borrows conservatively: once a borrow exists, later `move/free/write` on the same owned object are rejected until the borrow is ordered to end, either by last-use inference or by an explicit `cpu.borrow_end` node.
* `kernel.*` ops are the standard tensor/kernel execution surface. They may lower to `npu`, `gpu-kernel`, or future accelerators without changing the core graph semantics.
* `data.*` ops are the instruction-level surface for Fabric-style exchange. The architecture term `Fabric` remains valid, but the standard op family name is `data`.
* The current handwritten prototype now includes a first typed Fabric surface: `move`, `copy_window`, `immutable_window`, `marker`, `bind_core`, `output_pipe`, `input_pipe`, and `handle_table`.
* The current verifier already enforces a minimal legality set for that surface: `input_pipe` must consume `output_pipe`, nested pipes are rejected, and `window` wrappers may not be formed from marker/handle carriers.
* The current frontend/runtime contract now also distinguishes local mutable windows from bridge-safe immutable windows: `copy_window` is the local mutable-view primitive, while `immutable_window` is the stable cross-domain view primitive.
* Current project/data-pipe paths only allow immutable windows to cross the explicit `output_pipe/input_pipe` bridge; mutable windows are expected to stay local until they are converted into an immutable view.
* `data.move` is the current `MoveValue` primitive surface and therefore only accepts plain value payloads, not `window`, `marker`, `handle`, or `pipe` carriers.
* `data.move` also currently behaves as an `Own` consume in the verifier: after a move, later graph-visible uses of the same source must already be ordered before that move node.
* `data.handle_table` is strictly a resource-indirection primitive: entries must use unique slots and may only reference declared resources, never concrete data payloads.
* `data.bind_core` is the current CPU-hosted Fabric worker binding surface: it lets the AOT/bundle path record which CPU core the data-plane worker should occupy, in a DPDK-like style.
* These domain surfaces are expected to graduate into `nustar` registration packages; `nuisc` should load and bind them through a static index plus lazy package loading, rather than proactively discovering and hard-coding them as part of core YIR.

Resource kinds are intentionally open-ended. For example, the current macOS
window/backend path may eventually lower to Metal, but the YIR grammar does not
hard-code backend selection.

Current shader lowering contract direction:

* `shader.target + shader.pipeline + shader.begin_pass + shader.draw_instanced` form the current backend-lowerable render subset.
* `shader.uv / shader.texture2d / shader.sampler / shader.sample / shader.sample_uv` provide the current minimal texture-resource and normalized sampling surface.
* `shader.sample` and `shader.sample_uv` are the preferred sampling ops; they dispatch through `sampler.filter`.
* `shader.sample_nearest / shader.sample_uv_nearest / shader.sample_uv_linear` remain as compatibility aliases for reference/testing paths.
* `shader.const_bool / const_i32 / const_i64 / const_f32 / const_f64` and `shader.add_*/mul_*` are the current typed-scalar surface inside the shader family.
* `shader.draw_ball / shader.draw_sphere / shader.draw_instanced` now accept typed scalar packets directly. The current reference path accepts tuple packets like `(color, speed[, radius_scale])` and struct packets with at least `color` and `speed`.
* `shader.vertex_layout / shader.vertex_buffer / shader.index_buffer` provide the current minimal geometry-input surface.
* `shader.uniform / shader.storage / shader.attachment / shader.texture_binding / shader.sampler_binding / shader.vertex_layout_binding / shader.vertex_binding / shader.index_binding / shader.bind_set` are the current resource-layout surface around that subset.
* `shader.blend_state / shader.depth_state / shader.raster_state / shader.render_state` are the current minimal render-state surface around that subset.
* The handwritten reference renderer already consumes the bound geometry surface for vertex placement, primitive edge drawing, and minimal triangle-area coverage for `triangle` / `triangle_strip`.
* This subset is intended to map to common `Metal/Vulkan` concepts, not to either backend's source language directly.
* Legacy reference ops such as `shader.draw_ball`, `shader.draw_sphere`, and generic `shader.dispatch` remain valid YIR, but currently fall back to prerender/reference execution rather than entering the portable backend subset.
* `kernel.const_bool / const_i32 / const_i64 / const_f32 / const_f64` and `kernel.add_*/mul_*` are the current typed-scalar surface inside the kernel family.
* `kernel.fill` now accepts either a direct integer literal or a typed scalar value reference.
* `kernel.add_bias` now accepts either a bias tensor or a typed scalar value reference.
* `kernel.splat / add_scalar / mul_scalar` are the current tensor-scalar bridge for the integer tensor surface.
* `kernel.shape / rows / cols / row / col / element_at / reshape / slice / broadcast` are the current minimal shape/index/transform surface.
* `kernel.reduce_sum_axis / reduce_max_axis / reduce_mean_axis / reduce_min_axis / argmax_axis / argmin_axis` are the current minimal axis-reduction surface, with `rows` and `cols` as the first supported reduction directions.
* `kernel.reduce_max / reduce_mean / reduce_min / argmax / argmin` now complement the existing global `reduce_sum`.
* `kernel.sort / topk / topk_axis` are the current minimal selection surface.
* `kernel.sort / topk` currently operate on the flattened tensor payload and return a `1xN` result tensor.
* `kernel.topk_axis rows|cols` returns per-row or per-column top-k values in descending order.
* `kernel.add / mul / add_bias` now perform automatic shape alignment when the inputs are broadcast-compatible.
* Full typed tensor ABI is still intentionally deferred.
* Package-level deployment should treat backend outputs as per-stage variants under one semantic stage id, so the same YIR stage can later carry `Metal`, `Vulkan`, `DirectX`, or `OpenGL` artifacts without changing the core graph.

Cross-domain exchange is represented as a dedicated edge kind:

```text
edge xfer <from> <to>
```

That keeps domain crossing explicit in the graph instead of burying it in
instruction order.

For the current direct shader-driven and CPU-hosted preview demo direction, one optional adapter path expresses:

```text
cpu-hosted window + input sample
    ->
cpu-side state build
    ->
cross-domain exchange
    ->
shader-side render packet + draw
    ->
host-side frame presentation
```

### Semantics

```
compute(value...) → value
```

Pure computation is side-effect free.

Effects must be explicitly represented.

---

### Resource

Resources represent execution units or devices.

Examples:

```
CPU core
GPU device
accelerator unit
```

YIR controls execution scheduling over these resources.

---

# 4. Data Fabric IR (Fabric IR)

Fabric IR represents data exchange between execution units.

Fabric IR is a **typed static dataflow fabric graph**.

```
Fabric IR = typed pipe network
```

---

## 4.1 Fabric Primitives

Fabric IR consists of seven primitives.

| Primitive             | Meaning              |
| --------------------- | -------------------- |
| Move Value            | transfer ownership   |
| Copy Window           | duplicate data view  |
| Immutable Window      | read-only data view  |
| Phantom Marker        | logical boundary     |
| Input Pipe            | fabric ingress       |
| Output Pipe           | fabric egress        |
| Resource Handle Table | resource indirection |

These primitives form the minimal algebra for data exchange.

---

# 5. Pipe System

Pipes are typed channels connecting units.

```
Pipe<T>
```

A pipe represents a compile-time dataflow edge.

Example:

```
Output Pipe<Window<f32>>
      ↓
Input Pipe<Window<f32>>
```

Verifier enforces type compatibility.

---

# 6. Window Model

Window represents a data view.

```
Window =
    base
    offset
    shape
    stride
```

Windows may be nested and may span multiple devices.

Examples:

```
matrix tile
tensor slice
packet segment
image block
```

Windows do not define topology; they describe layout and slicing.

---

# 7. Type System

Pipe types may use primitive-derived generics.

Allowed constructions:

```
Value
Window<T>
Handle<Resource>
Marker<Tag>
Tuple<T...>
```

Types must ultimately be composed from primitives.

User-defined arbitrary structures are not allowed in Fabric IR.

This ensures verifier tractability.

---

# 8. Verifier

All Nustar modules must provide a verifier.

The verifier performs dataflow correctness validation.

Verifier responsibilities include:

### Type Safety

```
Pipe type compatibility
```

### Ownership Flow

```
Move semantics correctness
```

### Window Validity

```
window bounds
stride legality
```

### Resource Lifetime

```
handle table correctness
```

### Graph Legality

```
pipe connectivity
unit compatibility
```

Verifier must guarantee that the IR graph is semantically valid before lowering.

---

# 9. Lowering Model

The current in-repo implementation target is intentionally conservative:

```
YIR → AOT compute lowering
Fabric IR → AOT data-plane lowering
```

Data-plane workers execute compiled data movement pipelines.

This model follows a philosophy similar to data-plane systems such as DPDK-style pipelines.

Future hardware may provide dedicated fabric execution units, but that does not change the AOT-first semantic contract defined here.

---

# 10. Extensibility (Nustar)

A Nustar module may define:

```
execution units
lowering rules
verifier rules
```

However, Nustar modules **may not introduce new primitives**.

All semantics must be expressed through composition of existing primitives.

---

# 11. Version Scope

Version 0.01 defines:

```
primitive semantics
dataflow model
type model
verifier responsibilities
```

Future versions may extend lowering strategies and optimization models.

Primitive set stability is strongly preferred.

---
