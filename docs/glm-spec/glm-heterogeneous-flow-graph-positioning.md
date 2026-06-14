# GLM As Heterogeneous Flow-Graph Semantics

This file states the intended role of `GLM` in `nuis`.

The short version is:

`GLM` is not only a borrow/lifetime technique.

It is the resource-semantics layer for a heterogeneous execution graph.

Ownership, borrowing, moves, and lifetime edges matter because they are the
most reliable current tools for expressing that graph semantics, not because
the project is trying to become a pure borrow-checker-first language.

## What GLM Is

`GLM` should be read as the layer that answers:

* what kind of thing is flowing here: `val`, `res`, observation handle,
  bridge-shaped object, or a future richer graph class
* what kind of use is happening: `Read`, `Write`, `Own`, or later stronger
  domain-specific modes
* what effects happen on the graph: consume, replace, lifetime end,
  observation, readiness probe, domain move
* what ordering is required before a node is legal
* which control-flow path facts make a later access valid or invalid

That means `GLM` is about graph-visible meaning first.

The ownership/lifetime vocabulary exists to make those meanings explicit and
verifiable.

## What GLM Is Not

`GLM` should not be read as only:

* a clone of a Rust-style borrow checker
* a local memory-safety checker for CPU pointers
* a task-only ownership rule set
* a replacement for the full concurrency model

Those are all related surfaces, but they are narrower than the real job.

If the repository later supports more domains, more bridge objects, or richer
cross-domain scheduling, `GLM` should still make sense as the same layer.

That is a strong signal that its real scope is the heterogeneous graph, not
just the CPU lifetime story.

## Why Lifetime Techniques Still Matter

The current repository already needs a way to say:

* this resource was consumed here
* this write cannot happen while a borrow path is active
* this later node is only legal if an earlier path established a completed
  observation state
* this handle crossing should not be treated like an unrestricted plain value

Borrow/move/lifetime techniques are useful because they provide:

* local verifier rules that are easy to explain
* graph-lowering hooks that are easy to preserve through `NIR -> YIR`
* explicit failure modes instead of silent semantic drift

So it is correct to say `GLM` is in the same family as lifecycle technology.

It is more correct to say lifecycle technology is one implementation strategy
inside a larger graph-semantics job.

## Core GLM Responsibilities

The current and future responsibilities of `GLM` should be grouped like this.

### 1. Resource Lifetime Discipline

This is the narrowest visible layer today.

It includes:

* `move`, `borrow`, `borrow_end`, `free`
* consume-vs-observe boundaries
* owned vs borrowed write restrictions
* expression-order-sensitive resource invalidation

Today this is the part most obviously visible in `nir_verify` and in the
current `YIR` lifetime-edge checks.

### 2. Observation And Result-Path Semantics

Heterogeneous domains already return stateful observation objects:

* `TaskResult<T>`
* `NetworkResult<T>`
* `KernelResult<T>`
* `ShaderResult<T>`

`GLM` should govern not just that they exist, but how they may be observed.

Examples:

* `task_value(result)` is only valid on a completed path
* readiness/value probes are not the same kind of use
* some result shapes are reusable observation handles, not immediate payload
  transfer objects

This is where `GLM` starts to become clearly more than a pointer-lifetime tool.

### 3. Control-Flow-Sensitive Legality

Graph legality depends on path facts, not only node-local typing.

So `GLM` should understand:

* `if` path refinement
* `while` path refinement
* short-circuit condition semantics
* left-to-right expression evaluation where earlier subexpressions can change
  the legality of later subexpressions

The current repository has already started moving in this direction through:

* task-result condition facts
* expression-order-sensitive verifier behavior
* borrow-sensitive write/consume rejection

That is the right direction, because heterogeneous graphs are path-sensitive by
nature.

### 4. Domain-Bridge Legality

The hardest `GLM` work is not local CPU memory.

It is the question:

* when a value crosses from one domain-facing interpretation to another, what
  is preserved, consumed, wrapped, reified, or forbidden?

This includes future bridge-shaped semantics for:

* external handles
* task/runtime boundaries
* host/device crossings
* fabric- and project-level object exchange

If this layer is not explicit, later compiler stages will either over-assume
or silently erase important semantics.

### 5. Lowering And Scheduling Preconditions

`GLM` is also a compiler contract layer.

It should tell later phases:

* which nodes are safe to lower as observation-only reads
* which edges must survive optimization
* which ordering constraints a scheduler must respect
* which graph rewrites would erase ownership/lifetime meaning and are therefore
  illegal

This is one reason `GLM` must be framed as flow-graph semantics, not as a
surface-language-only safety feature.

## Current Repository Shape

Today the repository is not yet at a full final `GLM`.

The checked-in implementation is better described as:

* a minimal explicit graph lifetime layer in `YIR`
* a stricter `NIR` verifier with ownership-sensitive and path-sensitive checks
* task/result/domain observations that are beginning to align with that layer

Current implementation examples:

* `YIR` lifetime edges for ownership-sensitive operations
* `nir_glm_profile(...)` classification
* path-sensitive `task_value(...)` checks
* expression-order-sensitive move/borrow verification

This is already meaningful.

It is still smaller than the final target.

## Safety And Verification Layers

The repository should not treat `GLM` as the only safety layer.

The better model is a stack of related but different layers.

### 1. Ownership And Address Rules

This is the narrowest local safety layer.

It answers questions like:

* is this resource still usable?
* is this address owned or borrowed?
* is this write legal while a borrow path is active?
* did this path already consume or free the object?

This layer is local and operational.

It gives the system a minimal truth about resource validity before higher graph
semantics are considered.

### 2. GLM

`GLM` sits above local ownership rules.

Its job is not to become the final proof engine.

Its job is to provide a generic graph-facing resource semantics layer that can
be shared across heterogeneous domains.

That means `GLM` should stay:

* relatively small in vocabulary
* reusable across domains
* expressive enough for lowering and verifier contracts
* stable enough to be consumed by later tooling

So `GLM` should be strong, but not overly heavy.

It should express the right graph facts, not every future proof obligation.

### 3. Compiler-Native YIR Verification

The compiler's own `YIR` verifier is the mainline contract guard.

Its responsibility is:

* after lowering, confirm that graph structure is still legal
* confirm required ordering/resource edges still exist
* confirm domain contracts and operation-shape rules were not lost
* enforce the compiler's own checked-in graph contract

This verifier is about keeping the compiler honest with itself.

It is not the final external analysis layer.

It is the internal guard that says:

* the lowered graph still satisfies the repository's current compile-time
  invariants

### 4. Vulpoya

`vulpoya` should be read as a later independent analysis layer.

Its role is closer to:

* a deep semantic analyzer
* a second-pass graph reviewer
* a future formal-verification entry point

It is analyzer-like in the sense that it is not the core compiler runtime
itself.

But it should be stronger than a typical editor helper because it can perform
deeper semantic and graph-level rechecks over stable interfaces.

The important boundary is:

* compiler-native `YIR` verification keeps the mainline contract closed
* `vulpoya` performs deeper independent secondary review

That secondary review can eventually include:

* richer graph consistency checks
* stronger cross-domain reasoning
* formal or semi-formal proof-oriented workflows
* tool-facing diagnostics outside the core compile path

### Why This Layering Matters

This layering keeps each system from becoming overloaded.

If `GLM` tries to carry every proof obligation:

* it becomes too heavy
* it stops being generic
* it becomes harder to preserve through lowering
* it becomes harder for external tools to consume cleanly

If compiler-native `YIR` verification tries to become the only deep analysis
engine:

* the core compile path becomes harder to evolve
* independent validation becomes harder

If `vulpoya` has no stable intermediate semantic layer to consume:

* external verification work becomes tightly coupled to compiler internals

So the intended stack is:

* ownership/address rules
* `GLM`
* compiler-native `YIR` verifier
* `vulpoya`

This should be read as a cooperating stack, not a set of competing systems.

`GLM` is the shared graph-semantics layer in the middle, not the entire
security story by itself.

## Current Design Rule

When adding new `GLM` work, prefer this question order:

1. what graph-visible semantic distinction are we trying to preserve?
2. what access/effect vocabulary is needed to express it?
3. what verifier and lowering rules are needed to keep it true?
4. only then ask whether the local syntax resembles borrow/lifetime machinery

This order matters.

If the project starts from local syntax alone, `GLM` risks collapsing into a
CPU-only ownership story.

If it starts from graph-visible meaning, lifetime rules stay aligned with the
heterogeneous system they are supposed to serve.

## Working Mental Model

For the current line, the safest mental model is:

* `GLM` is the graph contract that says which resources, observations, and
  bridge objects may flow where
* ownership/lifetime rules are the current main enforcement vocabulary
* heterogeneous result states and bridge semantics are not "extra features";
  they are central to why `GLM` exists

That is the framing future design docs should preserve.

## Relationship To Current Reference Docs

Read this file as the positioning layer.

For current implementation truth, cross-check with:

* [../reference/yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [../reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* [../reference/cpu-task-glm-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/cpu-task-glm-contract.md)
* [vulpoya-yir-secondary-review-positioning.md](/Users/Shared/chroot/dev/nuislang/docs/glm-spec/vulpoya-yir-secondary-review-positioning.md)

If a future design note and current verifier behavior disagree:

* trust checked-in verifier behavior for current truth
* narrow the design note rather than overclaiming
