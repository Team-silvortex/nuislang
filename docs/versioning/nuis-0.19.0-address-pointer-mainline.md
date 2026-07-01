# `nuis` 0.19.0 Address / Pointer Mainline

This file is the short implementation-facing map for the current address-type
story in the `0.19.*` line.

It follows the `0.18.*` decision that `ref` is already a real internal address
baseline.

The `0.19.*` addition is not a new pointer design.

It is a clearer source-style and documentation boundary on top of that same
baseline.

For the established `0.18.*` design context, also see:

* [nuis-0.18.0-address-pointer-mainline.md](nuis-0.18.0-address-pointer-mainline.md)
* [../reference/address-surface-contract.md](../../docs/reference/address-surface-contract.md)
* [../reference/nir-memory-model.md](../../docs/reference/nir-memory-model.md)

## Current Address Shapes

There are still two current address families.

Structural addresses:

* `ref Node`
* source surface: `ptr.value`, `ptr.next`, `ptr.value = value`, `ptr.next = next`
* builtin lowering core: `load_value(ptr)`, `store_value(ptr, value)`
* builtin lowering core: `load_next(ptr)`, `store_next(ptr, next)`

Buffer addresses:

* `ref Buffer`
* source surface: `buffer.len`, `buffer[index]`, `buffer[index] = value`
* builtin lowering core: `buffer_len(buffer)`, `load_at(buffer, index)`
* builtin lowering core: `store_at(buffer, index, value)`

## Current `0.19.*` Rule

The most important current rule is now:

* checked-in `.ns` source should prefer the surface spellings
* lowered builtin helper names remain the implementation-facing truth
* host-boundary pointer ABI is still narrow, but no longer purely `i64`-spelled:
  non-optional `ref Buffer` extern parameters now lower through the same
  buffer-handle bridge automatically

Short rule:

`0.19.*` is not changing the address core; it is making the current layer boundary explicit`

## Ownership Rule

Current verifier truth is still:

* owner values can be moved
* borrowed aliases cannot be moved
* read-only address access may flow through borrowed aliases
* writes, moves, and frees require owner authority
* host-boundary pointer ABI is still intentionally narrower than internal
  `ref` truth

## Current Front-Door Examples

Smallest structural pointer route:

* [hello_glm.ns](../../examples/ns/memory/hello_glm.ns)
* [hello_borrow_end.ns](../../examples/ns/memory/hello_borrow_end.ns)
* [hello_ref_struct.ns](../../examples/ns/types/hello_ref_struct.ns)

Smallest buffer address route:

* [hello_buffer_addressing.ns](../../examples/ns/memory/hello_buffer_addressing.ns)

## Current Boundary

`0.19.*` still does not claim:

* pointer arithmetic
* arbitrary address casts
* explicit surface-level owner-vs-borrow type separation
* stable general host ABI pointer transfer
* pointer returns across `extern`
* arbitrary `ref T` extern parameters
* generalized loop-memory write lowering

Current narrow host exception:

* `extern "c" fn foo(buffer: ref Buffer, ...) -> i64` is now accepted
* calls lower that `ref Buffer` argument through the existing host buffer-handle
  bridge
* other `ref` parameter types and all `ref` returns remain rejected

## Rule Of Thumb

Treat `0.19.*` address work as:

* one already-real internal `ref` baseline
* one now-explicit source-facing style contract
* one continued verifier/lowering truth below that surface
