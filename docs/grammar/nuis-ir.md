---

# Nuis Intermediate Representation

## Draft Specification v0.01

---

# 1. Overview

Nuis is an ahead-of-time (AOT) compiled language designed for heterogeneous computing systems.

The Nuis compilation pipeline separates program semantics into two orthogonal intermediate representations:

```
YIR   — Execution topology
DFIR  — Data fabric topology
```

These two graphs remain independent and interact through typed channels.

```
Program =
ExecutionGraph (YIR)
×
DataFabricGraph (DFIR)
```

This separation enables independent optimization of computation and data movement.

---

# 2. Core Principles

### 2.1 Orthogonality

Execution and data movement are modeled independently.

```
compute ≠ data movement
```

YIR describes **how computation executes**.

DFIR describes **how data moves between units**.

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

DFIR primitives are fixed.

Extensions must **compose primitives** rather than introduce new ones.

This guarantees verifier tractability.

---

### 2.4 Immutable-First Data Model

Persistent data is immutable.

Mutability is only allowed within explicitly bounded transient stages.

---

### 2.5 Hardware Independence

The IR is not specialized for any specific hardware.

Current implementations may lower to CPU cores, but the model is designed to support future dedicated fabric hardware.

---

# 3. Execution IR (YIR)

YIR represents execution topology.

It describes computation, synchronization, and resource usage.

### Core Operations

```
compute
move
sync
effect
resource
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

# 4. Data Fabric IR (DFIR)

DFIR represents data exchange between execution units.

DFIR is a **typed static dataflow fabric graph**.

```
DFIR = typed pipe network
```

---

## 4.1 DFIR Primitives

DFIR consists of seven primitives.

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

User-defined arbitrary structures are not allowed in DFIR.

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

Current implementation lowers the system as follows:

```
YIR → compute cores
DFIR → fabric worker cores
```

Fabric workers execute compiled data movement pipelines.

This model follows a philosophy similar to data-plane systems such as DPDK-style pipelines.

Future hardware may provide dedicated fabric execution units.

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
