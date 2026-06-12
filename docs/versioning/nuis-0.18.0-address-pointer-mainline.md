# Nuis 0.18.0 Address / Pointer Mainline

This file is the short implementation-facing map for the current address-type
story.

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
* `load_value(ptr)`
* `store_value(ptr, value)`
* `load_next(ptr)`
* `store_next(ptr, next)`

Buffer addresses:

* `ref Buffer`
* `alloc_buffer(len, fill)`
* `load_at(buffer, index)`
* `store_at(buffer, index, value)`

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
* owner writes during active borrows are rejected
* borrowed pointers cannot be written into structural `next` links

This is the current pointer-safety contract:

* [docs/reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

## Current Front-Door Examples

Smallest structural pointer route:

* [hello_glm.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_glm.ns)
* [hello_borrow_end.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_borrow_end.ns)
* [hello_ref_struct.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/types/hello_ref_struct.ns)

Smallest buffer address route:

* [hello_buffer_addressing.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_buffer_addressing.ns)

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
