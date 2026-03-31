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

Current built-in / registered examples:

```text
cpu.const <name> <resource> <value>
cpu.null <name> <resource>
cpu.borrow <name> <resource> <ptr>
cpu.move_ptr <name> <resource> <ptr>
cpu.add <name> <resource> <lhs> <rhs>
cpu.sub <name> <resource> <lhs> <rhs>
cpu.mul <name> <resource> <lhs> <rhs>
cpu.madd <name> <resource> <lhs> <rhs> <acc>
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
cpu.target_config <name> <resource> <arch> <abi> <vector_bits>
cpu.bind_core <name> <resource> <core_index>
cpu.window <name> <resource> <width> <height> <title>
cpu.input_i64 <name> <resource> <channel> <default>
cpu.present_frame <name> <resource> <frame>
cpu.print <name> <resource> <input>
kernel.target_config <name> <resource> <arch> <runtime> <lane_width>
kernel.tensor <name> <resource> <rows> <cols> <csv-elements>
kernel.fill <name> <resource> <rows> <cols> <value>
kernel.matmul <name> <resource> <lhs> <rhs>
kernel.add_bias <name> <resource> <input> <bias>
kernel.relu <name> <resource> <input>
kernel.print <name> <resource> <input>
data.move <name> <resource> <input> <to>
shader.const <name> <resource> <value>
shader.add <name> <resource> <lhs> <rhs>
shader.mul <name> <resource> <lhs> <rhs>
shader.target <name> <resource> <format> <width> <height>
shader.viewport <name> <resource> <width> <height>
shader.pipeline <name> <resource> <shading_model> <topology>
shader.pack_ball_state <name> <resource> <color> <speed>
shader.begin_pass <name> <resource> <target> <pipeline> <viewport>
shader.dispatch <name> <resource> <input>
shader.draw_instanced <name> <resource> <pass> <packet> <vertex_count> <instance_count>
shader.draw_ball <name> <resource> <packet>
shader.draw_sphere <name> <resource> <packet>
shader.print <name> <resource> <input>
```

Important boundary note:

* `cpu.window`, `cpu.input_i64`, and `cpu.present_frame` are not YIR-core UI semantics.
* They are current `cpu`-mod extension ops used by the reference preview/runtime path.
* A different frontend, runtime adapter, or future framework can consume the same YIR graph without depending on these specific ops.
* `cpu.borrow` and `cpu.move_ptr` are the first Rust-like ownership surface for the pure CPU domain: reads may flow through borrowed pointers, while writes and frees remain ownership-sensitive.
* `cpu.alloc_node / alloc_buffer / load_* / store_* / free` are an early reference prototype for addressable objects and pointer-like semantics. They are intentionally narrow and currently model controlled heap-node and heap-buffer surfaces rather than a full general memory model.
* `kernel.*` ops are the standard tensor/kernel execution surface. They may lower to `npu`, `gpu-kernel`, or future accelerators without changing the core graph semantics.
* `data.*` ops are the instruction-level surface for Fabric-style exchange. The architecture term `Fabric` remains valid, but the standard op family name is `data`.
* These domain surfaces are expected to graduate into `nustar` registration packages; `nuisc` should discover and bind them as registered capability bundles rather than hard-coding them as part of core YIR.

Resource kinds are intentionally open-ended. For example, the current macOS
window/backend path may eventually lower to Metal, but the YIR grammar does not
hard-code backend selection.

Current shader lowering contract direction:

* `shader.target + shader.pipeline + shader.begin_pass + shader.draw_instanced` form the current backend-lowerable render subset.
* This subset is intended to map to common `Metal/Vulkan` concepts, not to either backend's source language directly.
* Legacy reference ops such as `shader.draw_ball`, `shader.draw_sphere`, and generic `shader.dispatch` remain valid YIR, but currently fall back to prerender/reference execution rather than entering the portable backend subset.
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
