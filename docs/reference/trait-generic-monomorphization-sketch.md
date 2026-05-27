# Trait And Generic Monomorphization Sketch

This document records a narrow first design for trait-constrained generics in
`nuis`.

It is intentionally a **small implementation sketch**, not a claim that the
repository already supports traits today.

The goal is to answer one concrete question:

* how should `nuis` grow from existing generic type references like `Task<T>`
  and `Pipe<T>` into a first real layer of
  `trait / impl / constrained generic fn`?

## Why This Matters Now

The repository now has:

* generic-looking type references in the AST/NIR type model
* many stdlib and domain lanes that are starting to want reusable abstraction
* growing pressure from algorithm-shaped examples and more realistic stdlib
  APIs

But it still does **not** have:

* `trait`
* `impl`
* constrained generic functions
* a monomorphization or dictionary-passing story

That gap is beginning to matter because otherwise the stdlib tends to grow by:

* naming convention
* copy-pasted helpers
* increasingly specialized recipe ladders

instead of by explicit reusable constraints.

## Current Ground Truth

Today the repository already has:

* generic type arguments on AST types
* generic type arguments on NIR types
* front-end lowering for existing built-in generic families

Relevant anchors:

* [model.rs](/Users/Shared/chroot/dev/nuislang/crates/nuis-semantics/src/model.rs)
* [parser.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/parser.rs)
* [mod.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/mod.rs)

This means the most important missing piece is not angle-bracket parsing by
itself. The missing piece is:

* semantic constraint modeling
* trait member lookup
* generic function instantiation

## First-Cut Goal

The first supported surface should be only:

```ns
trait Addable {
  fn add(lhs: Self, rhs: Self) -> Self;
}

impl Addable for i64 {
  fn add(lhs: i64, rhs: i64) -> i64 {
    return lhs + rhs;
  }
}

fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
  return Addable.add(lhs, rhs);
}
```

This is intentionally smaller than Rust.

The design target is:

* trait declaration
* impl declaration
* one-trait generic constraint on a function
* compile-time monomorphization
* static trait method resolution

## First-Cut Non-Goals

The first version should explicitly **not** support:

* trait objects or `dyn`
* default trait methods
* associated types
* blanket impls
* overlapping impls
* negative impls
* multi-trait bounds like `T: A + B`
* `where` clauses
* trait methods that close over runtime vtables/dictionaries
* generic structs or generic impl blocks beyond the minimum needed for
  constrained functions

This keeps the first move small enough to fit the current compiler shape.

## Why Monomorphization First

There are two broad strategies:

### 1. Dictionary Passing

This would thread explicit trait implementation dictionaries through calls.

Why it is attractive:

* conceptually flexible
* closer to interface-like runtime dispatch models

Why it is a poor first move here:

* pushes new runtime value shapes into NIR/YIR too early
* complicates lowering and ABI surfaces
* interacts badly with the repository’s current preference for narrow,
  explicit effects and value carriers

### 2. Monomorphization

This means:

* resolve the impl statically
* clone the generic function per concrete use
* lower the cloned concrete function as if it were handwritten

Why it is the better first move here:

* matches the current ahead-of-time pipeline
* avoids runtime dictionary objects
* keeps YIR mostly concrete
* lets the first trait layer ride on top of existing concrete lowering

Current design judgment:

* first version should use **compile-time monomorphization**

## Proposed First Syntax

### Trait Declarations

```ns
trait Addable {
  fn add(lhs: Self, rhs: Self) -> Self;
}
```

Rules:

* methods are signatures only in the trait block
* `Self` is only valid inside trait and impl method signatures in v1
* no default method bodies in v1

### Impl Declarations

```ns
impl Addable for i64 {
  fn add(lhs: i64, rhs: i64) -> i64 {
    return lhs + rhs;
  }
}
```

Rules:

* the impl target must be a concrete type in v1
* the method set must exactly match the trait signature set
* no generic impls in v1

### Constrained Generic Functions

```ns
fn sum_two<T: Addable>(lhs: T, rhs: T) -> T {
  return Addable.add(lhs, rhs);
}
```

Rules:

* one generic parameter is enough for the first move
* one trait bound is enough for the first move
* trait method invocation should prefer explicit trait-qualified form first:
  `Addable.add(lhs, rhs)`

This avoids having to solve full method receiver inference on day one.

## AST / NIR Additions

The smallest new nodes are likely:

### AST

* `AstTraitDef`
* `AstTraitMethodSig`
* `AstImplDef`
* `AstGenericParam`
* generic params on `AstFunction`

### NIR

* `NirTraitDef`
* `NirTraitMethodSig`
* `NirImplDef`
* `NirGenericParam`
* a concrete “trait method call before monomorphization” form or an earlier
  front-end rewrite into resolved calls

Important design boundary:

* YIR should ideally never need a first-class generic or trait concept in the
  first implementation
* the monomorphization pass should erase generic/trait surface before normal
  CPU lowering

## Suggested Pipeline Shape

The narrowest workable sequence is:

1. parse trait/impl/generic-fn syntax
2. lower into AST/NIR semantic nodes
3. collect trait table
4. collect impl table
5. resolve constrained generic call sites
6. monomorphize generic functions into concrete functions
7. rewrite trait-qualified method calls into concrete callee calls
8. continue through existing NIR optimization and YIR lowering

This suggests a new compiler phase boundary:

* `generic_resolve`
* or `monomorphize_nir_module`

between front-end semantic lowering and existing optimization/lowering.

## Coherence Rules For V1

To keep the first version boring and predictable:

* one trait name may have only one impl for a given concrete type
* duplicate impls are an error
* missing trait methods are an error
* extra trait methods are an error
* generic function instantiation must fully resolve at compile time

No attempt should be made to support partial ambiguity.

## Call Surface Recommendation

The repository should start with explicit trait-qualified calls:

```ns
return Addable.add(lhs, rhs);
```

instead of implicit receiver-style:

```ns
return lhs.add(rhs);
```

Why:

* smaller parser change
* simpler trait lookup
* avoids first-pass ambiguity with existing field/method-like surfaces
* easier to rewrite during monomorphization

Receiver-style trait calls can be added later if the first layer succeeds.

## First Validation Sample

The first canonical sample should be something tiny like:

```ns
trait Step {
  fn step(value: Self) -> Self;
}

impl Step for i64 {
  fn step(value: i64) -> i64 {
    return value + 1;
  }
}

fn advance<T: Step>(value: T) -> T {
  return Step.step(value);
}

fn main() -> i64 {
  return advance<i64>(4);
}
```

This is intentionally better than jumping straight to collection algorithms,
because it isolates:

* trait parsing
* impl lookup
* monomorphized call generation

without dragging in larger control-flow questions.

## First Stdlib Targets

If v1 works, the most useful early wins would likely be:

* arithmetic-like helper traits for algorithm probes
* formatting/summary-like traits for stdlib reporting surfaces
* result/session summarization helpers

The first version should **not** start by trying to genericize the full network
or HTTP stack.

## Recommended Implementation Order

1. add AST syntax for trait/impl/generic-fn
2. add NIR model nodes
3. add rendering and verification stubs
4. add monomorphization pass for one generic param + one trait bound
5. add one canonical trait sample project
6. only then consider receiver-style sugar or broader bounds

## Short Version

The right first move is not:

* “full Rust trait system”

The right first move is:

* `trait`
* `impl Trait for ConcreteType`
* `fn f<T: Trait>(...)`
* explicit trait-qualified method calls
* compile-time monomorphization

That is small enough to fit the current compiler, and big enough to unlock more
normal standard-library growth.
