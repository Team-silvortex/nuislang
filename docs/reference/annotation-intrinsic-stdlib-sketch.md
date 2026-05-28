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

That should not be read as “surface syntax is the only truth”.
The current repository already leans on `nustar` registration and loader
contracts as the more stable semantic boundary.

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

## Replaceability Rule

Official annotations should be treated as the repository's preferred frontend
conventions, not as the only legal long-term surface spelling.

In practice:

* the official frontend may spell a packet contract as `@packet`
* another frontend could choose a different source spelling
* both are acceptable if they bind to the same registered capability surface
  and preserve the same validated downstream contract

The stable truth should live in:

* `nustar` registration metadata
* registration-time completeness / standards validation
* loader-contract requirements
* lowered capability contracts that downstream `YIR` and packaging consume

So the repository should optimize for:

* replaceable surface syntax
* replaceable `nustar` implementations
* stable registered capability contracts

This is especially important because platform packages such as an
`x86_64-cpu-linux`-style `nustar` are expected to be replaceable, as long as
their registration remains complete and standards-valid.

## Ownership Model

Annotations should be standard-library-owned in the same sense that other
current `std` growth paths are compiler-managed.

That means:

* annotation names are part of the std/compiler contract
* the compiler knows which annotations are valid on which targets
* invalid annotations are hard errors
* unknown annotations are hard errors
* lowering behavior is fixed by compiler intrinsic handling, not user hooks

But that ownership should still be read through the replaceability rule above:

* official annotations are std-owned frontend conventions
* they are not the sole source of semantic truth
* the stronger long-lived boundary is the registered capability contract
  carried through `nustar`

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
fn main() -> i64 {
  return 0;
}

extern "c" @host_symbol("network.open_tcp")
fn open_tcp(local_port: i64, remote_port: i64) -> i64;
```

This reads clearly and matches user expectations.

### Conservative Fallback: Std-Owned Marker Call Form

```ns
annotate(inline);
annotate(export(name = "main"));
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

Current host-boundary guidance:

* prefer `extern "c" @host_symbol("...") fn ...;` as the stable std-owned host boundary
* allow `@host_symbol("...") fn ... { return 0; }` only as an MVP bridge-stub form
* keep both forms compiler-owned and white-listed

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

Current first-step packet semantics should stay deliberately narrow:

* allow `@packet` on `struct`
* allow `@packet_field` on fields inside a packet struct
* surface packet-shape metadata through project/build indexes first
* include stable field order and coarse field-kind classification in that metadata
* distinguish coarse packet field roles such as `payload`, `control-plane`, `async-carrier`, and `unsupported-shape`
* treat `@packet_field` as a payload-slot marker for now
* treat `@packet_control_field` as an explicit control-plane slot marker for now
* surface a first encode skeleton through metadata, not generated code
  * `packet_encode_shape`
  * `payload_bytes`
  * `payload_layout`
  * per-field `wire_kind` / `fixed_width`
* start with narrow static packet-safety checks: packet structs must be non-empty, must declare at least one `@packet_field`, and currently reject `ref` / optional fields
* allow control-plane families like `Marker<...>` and `HandleTable<...>` only through `@packet_control_field`
* continue rejecting async-carrier families like `Task<...>` and `*Result<...>` for now
* do not jump straight to full encode/decode generation

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
* `nustar` registration remains the stronger replaceable capability boundary
  underneath that programming model

So the real model is:

`stdlib surface name + compiler intrinsic semantics`

not:

`ordinary library call + magical reflection later`

and not:

`single mandatory source spelling as the only semantic truth`

The shortest repository rule is:

`surface syntax is replaceable; registered capability contracts are the stable truth`

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
