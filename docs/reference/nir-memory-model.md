# NIR Memory Model

This file is the current implementation-facing reference for the `ref /
borrow / move / free` rules enforced by
[tools/nuisc/src/nir_verify.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/nir_verify.rs).

It is intentionally narrower than a future full runtime memory model. Right now
it describes the verifier contract that optimization and front-door tooling
should treat as real.

## Scope

Current `NIR` ownership checking is function-local and verifier-driven.

The important tracked states are:

* moved values
* active borrow counts per resource
* borrow-alias bindings such as `let head_ref = borrow(head)`

This is the layer that protects current CPU/CLI-style source code from obvious
ownership mistakes before lowering continues.

## Current Rules

### Owner values

* `move(x)` consumes `x`
* using `x` after that is rejected
* rebinding `x` while `x` still has active borrows is rejected

### Borrow aliases

* `borrow(x)` creates a borrow alias tied to `x`
* writing through the owner while any borrow of `x` is active is rejected
* moving a borrow alias is rejected
* rebinding a borrow alias while its borrow is still active is rejected

### Explicit borrow closure

* `borrow_end(alias)` closes the currently active borrow for that alias
* calling `borrow_end(...)` with no active borrow is rejected
* after a successful `borrow_end(alias)`, that alias is no longer treated as an
  active borrowed pointer by the verifier

### Structural pointer writes

Current `Node.next`-style structural pointer paths are protected:

* `alloc_node(..., borrowed_alias)` is rejected
* `store_next(target, borrowed_alias)` is rejected

This keeps borrowed pointers from being written into structural ownership links.

## Branching

Current branch merging is conservative:

* moved state merges across both branches
* active borrow counts merge across both branches

This means code after an `if` must remain valid even if the borrow/move happened
in only one branch.

## Current Boundary

This is not yet a full heap/allocator/runtime memory model.

In particular, current verifier semantics are still:

* function-local
* staging-oriented
* focused on obvious ownership and aliasing hazards

It is good enough to support current front-end examples, early CLI-oriented
code, and conservative compiler optimizations. It is not yet the final word on
cross-domain lifetime transfer, allocator policy, or full async ownership.
