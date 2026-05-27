# Annotation / Intrinsic Stdlib Sketch

This file sketches a deliberately narrow annotation model for `nuis`.

The goal is not “user-extensible metaprogramming”.
The goal is a compiler-managed, standard-library-owned annotation surface that
helps project organization, validation, and optimization stay predictable.

## Why This Model

For the current repository stage, a narrow annotation system is a better fit
than either:

* runtime reflection
* free-form macro expansion

Reflection would push more semantic truth into runtime surfaces that the current
toolchain does not naturally want.
Macros would make the language feel more flexible, but they would also make it
much harder to preserve a single compiler-owned truth for:

* project structure
* optimization boundaries
* host ABI lowering
* stdlib layering

The intended tradeoff is explicit:

* less user freedom
* more compiler control
* more stable optimization and tooling behavior

## Design Rule

Treat annotations as:

* structured attributes in the source language
* validated by the compiler
* interpreted only by white-listed compiler/std surfaces

Do not treat annotations as:

* runtime objects
* reflection metadata for arbitrary user inspection
* user-defined code generators
* procedural macros

## Ownership Model

Annotations should be standard-library-owned in the same sense that other
current `std` growth paths are compiler-managed.

That means:

* annotation names are part of the std/compiler contract
* the compiler knows which annotations are valid on which targets
* invalid annotations are hard errors
* unknown annotations are hard errors
* lowering behavior is fixed by compiler intrinsic handling, not user hooks

This keeps annotations in the same “managed truth” bucket as:

* host symbols
* task/result carriers
* network/std layering contracts

## First Syntax

Two syntax families are reasonable.

### Preferred: Attribute Form

```ns
@inline
@export(name = "main")
@host_symbol("network.open_tcp")
fn main() -> i64 {
  return 0;
}
```

This reads clearly and matches user expectations.

### Conservative Fallback: Std-Owned Marker Call Form

```ns
annotate(inline);
annotate(export(name = "main"));
annotate(host_symbol("network.open_tcp"));
fn main() -> i64 {
  return 0;
}
```

This is less elegant, but easier to stage if parser churn needs to stay low.

If a single direction is chosen, prefer the `@name(...)` attribute form.

## First Targets

First-version annotations should apply only to a small set of target kinds:

* function
* struct
* field
* module or file-level item

Avoid broader placement rules until the compiler contract is stable.

## First White-List

The first annotation set should stay small and compiler-owned.

### Optimization / Codegen Hints

* `@inline`
* `@noinline`
* `@cold`
* `@entry`

These should be hints, not arbitrary guarantees, except where the ABI contract
requires one.

### ABI / Host Boundary

* `@export(name = "...")`
* `@host_symbol("...")`
* `@abi("...")`

These should connect directly to existing lowering and packaging surfaces.

### Test / Tooling

* `@test`
* `@should_fail`
* `@timeout_ms(25)`

This is especially attractive because the repository already has compiler-aware
tooling and project front doors that benefit from a structured test surface.

### Std Intrinsic Families

These are deliberately not “open derive”.
They are compiler-recognized, std-owned intrinsic helpers.

* `@packet`
* `@profiled`
* `@kernel_entry`
* `@network_entry`

The exact first set can stay small, but the model should allow this style.

## What Not To Do In V1

Do not include these in the first version:

* user-defined annotations with custom lowering hooks
* procedural macros
* token rewriting
* runtime reflection over annotations
* arbitrary derive plugins
* attribute-controlled trait synthesis

Those all make the language more expressive, but they also pull the repository
toward a much larger metaprogramming surface than the current compiler maturity
really wants.

## Compiler Shape

The smallest useful compiler shape is:

* parser stores raw attributes on items
* frontend validates placement and argument shape
* frontend resolves raw attributes into a small validated annotation form
* lowering / optimizer / packaging consumes only validated annotations

Possible semantic model sketch:

* `AstAttribute`
* `AstAttributeArg`
* `NirAnnotation`

With this split:

* AST can preserve user spelling
* NIR only carries normalized, compiler-approved annotations

## Relationship To Stdlib

The user asked for this to be “part of the standard library”.

That should not mean annotations are ordinary runtime library functions.
It should mean:

* std defines the stable names and intent
* compiler owns validation and lowering
* docs present them as part of the std-facing programming model

So the real model is:

`stdlib surface name + compiler intrinsic semantics`

not:

`ordinary library call + magical reflection later`

## First Milestone

The most practical first milestone is:

1. add a generic attribute container to AST/NIR
2. support function-level attributes only
3. white-list:
   * `@inline`
   * `@noinline`
   * `@export(name = "...")`
   * `@host_symbol("...")`
   * `@test`
   * `@timeout_ms(...)`
4. make unknown attributes a hard error
5. lower only the ABI/codegen/tooling subset

That gives `nuis` a real annotation language without opening a macro system.

## Second Milestone

Once the container and validation rules are stable:

* add struct / field attributes
* add std intrinsic families like `@packet` or `@profiled`
* unify existing ad hoc compiler-aware markers under one annotation model

This is the point where annotations begin to improve project organization, not
just code generation.

## Main Recommendation

If `nuis` wants annotation syntax, the best fit is:

* compiler-recognized
* std-owned
* white-listed
* non-reflective
* non-macro

That is narrower than Rust macros and less dynamic than reflection, but it is
much easier to keep coherent with the current compiler/project model.
