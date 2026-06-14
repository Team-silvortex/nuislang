# Nuis 0.18.0 Address / Pointer Mainline

This file is the short implementation-facing map for the current address-type
story.

For the design comparison behind this current choice, also see:

* [nuis-0.18.0-address-surface-options.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-address-surface-options.md)
* [nuis-0.18.0-owned-borrowed-address-draft.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-owned-borrowed-address-draft.md)
* [../reference/address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)

The key rule is simple:

* current nuis already has a pointer core
* that core is spelled `ref ...`
* `0.18.0` should treat this as the real address baseline, not as an accidental
  side feature

## Current Address Shapes

There are two current address families.

Structural addresses:

* `ref Node`
* `null()`
* `is_null(...)`
* `alloc_node(value, next)`
* source surface: `ptr.value`, `ptr.next`, `ptr.value = value`, `ptr.next = next`
* builtin lowering core: `load_value(ptr)`, `store_value(ptr, value)`
* builtin lowering core: `load_next(ptr)`, `store_next(ptr, next)`

Buffer addresses:

* `ref Buffer`
* `alloc_buffer(len, fill)`
* source surface: `buffer.len`, `buffer[index]`, `buffer[index] = value`
* builtin lowering core: `buffer_len(buffer)`, `load_at(buffer, index)`
* builtin lowering core: `store_at(buffer, index, value)`

## Ownership Rule

`ref` is not just syntax sugar for “some nominal type”.

In current `NIR`, `ref` is a distinct type shape:

* [crates/nuis-semantics/src/model.rs](/Users/Shared/chroot/dev/nuislang/crates/nuis-semantics/src/model.rs)
* `AstTypeRef.is_ref`
* `NirTypeRef.is_ref`
* `NirTypeShape::Ref`

Current verifier truth is:

* owner values can be moved
* borrowed aliases cannot be moved
* `load_next(...)` inherits address authority from its input
* read-only address access remains available through borrowed aliases
* owner writes during active borrows are rejected
* borrowed pointers cannot be written into structural `next` links

Current surface-syntax rule is still intentionally narrow:

* owner pointers and borrow aliases both appear as `ref T`
* the distinction is currently verifier/state driven, not surface-type driven
* `borrow(ptr)` keeps the same `ref T` surface type
* `borrow_end(alias)` returns `Unit`, not a new pointer wrapper type
* ordinary `.ns` source should now prefer field/index surface syntax instead of
  spelling the builtin load/store helpers directly

This is the current pointer-safety contract:

* [docs/reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

More concretely, current read/write policy is:

* `load_value(ptr)` may read from owned or borrowed structural addresses
* `load_next(ptr)` may read from owned or borrowed structural addresses
* `load_at(buffer, index)` and `buffer_len(buffer)` may read from owned or borrowed buffer addresses
* `store_value`, `store_next`, `store_at`, `move`, and `free` require owner authority

Current front-end sugar now lives in:

* [../reference/address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md)

Shortest current rule:

* `ref` surface syntax is preferred where it lowers cleanly to the existing builtin address core

## Current Front-Door Examples

Smallest structural pointer route:

* [hello_glm.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns)
* [hello_borrow_end.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns)
* [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)

Smallest buffer address route:

* [hello_buffer_addressing.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns)

Current repository source-level truth:

* checked-in `.ns` examples and stdlib routes now use the surface spellings
* builtin helper names remain the implementation-facing contract for lowering,
  verifier rules, NIR discussion, and YIR/CPU documentation

## 0.18.0 Mainline Rule

For `0.18.0`, address work should follow this order:

1. keep `ref` as the canonical current address surface
2. strengthen compile and lowering gates for both `ref Node` and `ref Buffer`
3. keep borrow/owner rules explicit and conservative
4. only then consider whether a future explicit `Address<T>` / `Ptr<T>` surface
   is worth adding

## What Is Not There Yet

Current nuis does not yet expose a full raw-pointer system.

Notably absent:

* pointer arithmetic
* arbitrary address casts
* explicit distinction between owned pointer type and borrowed pointer type at
  the surface syntax level
* generalized heap policy / allocator strategy
* stable cross-domain pointer transfer semantics

So the right near-term framing is:

* `ref` already is the current address type family
* the next job is to systematize it
* future explicit pointer syntax should be layered on top of this verified core

## Compile Gates

Current checked gates for the address baseline now live in:

* [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)

They intentionally cover:

* structural pointer allocation
* structural next-link loads
* explicit borrow closure before owner write
* ref-carrying struct fields
* buffer allocation and indexed address reads/writes
