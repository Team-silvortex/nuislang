# Vulpoya As YIR Secondary Review

This file states the intended role of `vulpoya`.

The short version is:

`vulpoya` should not replace the compiler's own verifier.

It should act as an independent analyzer / verifier project that consumes a
stable graph-facing interface, with `YIR` as the primary current candidate for
that interface.

## What Vulpoya Is

`vulpoya` should be read as:

* a separate future analyzer/verifier project
* a deeper secondary review layer over compiler-emitted graph semantics
* a future formal-verification-friendly entry point

Its natural comparison point is analyzer-like tooling, but with stronger graph
and semantic ambitions than an editor-only helper.

So the best current mental model is:

* analyzer-like in deployment shape
* verifier-like in semantic depth
* proof-friendly in long-range direction

## What Vulpoya Is Not

`vulpoya` should not be read as:

* a replacement for the core compiler
* a replacement for the compiler-native `YIR` verifier
* a justification for weakening compile-time mainline contracts
* a reason to make `GLM` compiler-internal and tool-hostile

Those would collapse the intended layering.

## Why YIR Matters Here

`vulpoya` needs a stable enough semantic surface to consume.

`YIR` is the best current candidate because it is already:

* graph-shaped
* explicit about operations and edges
* close to lowering truth
* already carrying part of the repository's `GLM` semantics

That makes `YIR` suitable as the current review boundary for:

* graph legality
* resource ordering
* domain interaction shape
* observation / readiness / payload extraction structure

This does not mean `YIR` must be the final forever-only interface.

It means current repository design should keep `YIR` clean enough that a
separate analysis project could consume it without depending on fragile
compiler internals.

## Relationship To Compiler-Native YIR Verification

There are two different `YIR` verification roles.

### 1. Compiler-Native YIR Verification

This is the verifier that lives with the compiler.

Its job is:

* defend the compiler's own lowering contract
* confirm checked-in graph invariants after lowering
* reject graph shapes the compiler itself must never emit or accept

This layer is part of the mainline compile path.

It keeps the compiler honest with itself.

### 2. Vulpoya Secondary Review

`vulpoya` should sit after that layer, not instead of it.

Its job is:

* re-read graph semantics independently
* perform deeper or more exploratory checks
* surface stronger diagnostics without overloading the core compiler
* become a natural home for heavier formal or semi-formal verification work

So the intended relationship is:

* compiler-native verifier closes the mainline contract
* `vulpoya` performs independent secondary review on top of that contract

This is closer to "second-pass semantic review" than to "alternate compiler."

## Relationship To GLM

`GLM` and `vulpoya` should not be collapsed into one thing.

`GLM` is the generic graph-semantics layer.

`vulpoya` is one future consumer of that layer.

That means:

* `GLM` should stay generic and reusable
* `vulpoya` can perform stronger reasoning using `GLM`-shaped facts
* `GLM` does not need to carry every final proof obligation itself

This is an important design rule.

If `GLM` becomes too heavy, it stops being a clean intermediate semantic layer.

If `vulpoya` has no stable `GLM`/`YIR`-shaped interface to consume, it becomes
tightly coupled to compiler internals.

The healthy split is:

* `GLM` expresses graph-facing resource semantics
* `YIR` carries those semantics in an explicit graph form
* `vulpoya` performs deeper independent analysis over that interface

## Likely Vulpoya Responsibilities

The current repository should reserve `vulpoya` for work such as:

* deeper consistency checks than the main compile path should own
* richer cross-domain graph review
* independent resource/ordering sanity review
* stronger secondary diagnostics for graph misuse
* future formal-verification-oriented passes over graph semantics

This makes it a good place for work that is:

* valuable
* semantically heavy
* too expensive, too experimental, or too specialized for the core compiler

## Design Rule For Today

When changing compiler, `GLM`, or `YIR` design, prefer choices that preserve a
clean external review boundary.

That means:

* keep graph semantics explicit rather than implicit
* keep `GLM` vocabulary stable where possible
* avoid lowering tricks that erase important ownership/ordering meaning
* avoid making the core verifier the only place where semantic truth can be
  reconstructed

If the repository does that well, `vulpoya` can stay independent while still
being powerful.

## Practical Reading

Read this file together with:

* [glm-heterogeneous-flow-graph-positioning.md](glm-heterogeneous-flow-graph-positioning.md)
* [../reference/yir-langref.md](../../docs/reference/yir-langref.md)
* [../reference/cpu-task-glm-contract.md](../../docs/reference/cpu-task-glm-contract.md)

If a future `vulpoya` plan and current repository behavior disagree:

* trust current compiler/reference behavior for present truth
* narrow the positioning note instead of overclaiming current capability
