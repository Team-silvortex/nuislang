# NIR Memory Model

This file is the current implementation-facing reference for the `ref /
borrow / move / free` rules enforced by
[tools/nuisc/src/nir_verify.rs](../../tools/nuisc/src/nir_verify.rs).

It intentionally names builtin read/write forms such as `load_value(...)` and
`store_at(...)` because this document describes verifier/NIR truth after
surface-syntax lowering.

For preferred ordinary `.ns` source spelling, see
[address-surface-contract.md](address-surface-contract.md).

It is intentionally narrower than a future full runtime memory model. Right now
it describes the verifier contract that optimization and front-door tooling
should treat as real.

## Scope

Current `NIR` ownership checking is function-local and verifier-driven.

The important tracked states are:

* moved values
* active borrow counts per resource
* borrow-alias bindings such as `let head_ref = borrow(head)`
* borrowed-derived traversal aliases such as `let next_ptr = load_next(head_ref)`

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
* `load_next(borrowed_alias)` preserves borrowed authority instead of regaining
  owner authority

### Explicit borrow closure

* `borrow_end(alias)` closes the currently active borrow for that alias
* calling `borrow_end(...)` with no active borrow is rejected
* after a successful `borrow_end(alias)`, that alias is no longer treated as an
  active borrowed pointer by the verifier

### Structural pointer writes

Current `Node.next`-style structural pointer paths are protected:

* `alloc_node(..., borrowed_alias)` is rejected
* `store_value(borrowed_alias, value)` is rejected
* `store_next(borrowed_alias, next)` is rejected
* `store_next(target, borrowed_alias)` is rejected
* `free(borrowed_alias)` is rejected

This keeps borrowed pointers from being written into structural ownership links.

### Buffer writes

Current `ref Buffer` write paths follow the same owner-authority rule:

* `store_at(borrowed_alias, index, value)` is rejected
* `free(borrowed_alias)` is rejected

This keeps borrowed buffer aliases readable but not writable/consumable.

### Read-only address access

Current read-only address operations remain available through borrowed aliases:

* `load_value(ptr)` may read from owned or borrowed structural addresses
* `load_next(ptr)` may read from owned or borrowed structural addresses
* `load_at(buffer, index)` may read from owned or borrowed buffer addresses
* `buffer_len(buffer)` may read from owned or borrowed buffer addresses

Short rule:

* reads may flow through borrowed aliases
* writes, moves, and frees require owner authority

## Branching

Current branch merging is conservative:

* moved state merges across both branches
* active borrow counts merge across both branches

This means code after an `if` must remain valid even if the borrow/move happened
in only one branch.

Practical consequences:

* if one branch borrows `head` and the other does not, a later owner write to
  `head` is still rejected unless all branches explicitly close the borrow
* if one branch moves `head` and the other does not, later reads of `head` are
  still rejected
* branch-local `borrow_end(...)` only restores owner writes after the verifier
  sees the borrow closed on every path that carried it

## Loops

Current loop handling is also conservative.

The verifier checks the loop body against cloned pre-loop state and then
conservatively merges body effects back with the pre-loop state to account for
either zero iterations or one-or-more iterations.

Practical loop rule:

* a borrow created inside the loop body is treated as possibly surviving the
  loop unless the body also closes it before the merged post-loop state
* a `borrow_end(...)` that happens only inside the loop body does not prove the
  loop ran, so it does not by itself restore pre-loop owner authority
* a move that occurs in the loop body is treated as possibly having happened
  after the loop
* a borrow created and closed entirely inside the loop body can still be safe
  after the loop because the merged post-loop borrow count returns to zero

Current control-flow reading:

* `if` merges are path-conservative
* `while` effects are iteration-conservative
* later owner writes or reads must remain valid under the most restrictive
  surviving state

Short rule:

* control flow never reintroduces owner authority by accident
* owner recovery must be explicit enough for the verifier to observe it without
  assuming a loop body definitely executed

## Current Boundary

This is not yet a full heap/allocator/runtime memory model.

In particular, current verifier semantics are still:

* function-local
* staging-oriented
* focused on obvious ownership and aliasing hazards

It is good enough to support current front-end examples, early CLI-oriented
code, and conservative compiler optimizations. It is not yet the final word on
cross-domain lifetime transfer, allocator policy, or full async ownership.

Important current split:

* verifier ownership semantics for `while` are ahead of minimal loop lowering
* some memory/address `while` shapes are now verifier-valid but still rejected
  by lowering until broader iterative backedge support exists
* guarded `while` bodies can already lower a small read-only memory subset such
  as `load_value(...)` / `load_at(...)` when the loop is reduced to guarded
  terminal forms like `return`
* chained/counting loop lowering can now encode fixed read-only carry-update
  sources such as loop-invariant `load_value(...)` and loop-invariant
  `load_at(buffer, index)` when the structural address / buffer / index do not
  depend on loop-variant state
* verifier now names `load_value(...)` and `load_at(...)` as fixed readable
  carry-source candidates, but verifier still only checks authority/readability;
  loop-invariance remains a prepare/lowering contract check
* chained/counting loop lowering still rejects broader memory-read carry
  expressions, including loop-variant indices, structural traversal through
  changing addresses, and write-capable backedge memory effects

## Current Checked Anchors

The current implementation truth is guarded by:

* verifier rules in
  [tools/nuisc/src/nir_verify.rs](../../tools/nuisc/src/nir_verify.rs)
* verifier regressions in
  [tools/nuisc/src/nir_verify/tests.rs](../../tools/nuisc/src/nir_verify/tests.rs)
  including conditional and loop ownership/borrow regressions
* address-type classification in
  [tools/nuisc/src/frontend/tests_types_async_window.rs](../../tools/nuisc/src/frontend/tests_types_async_window.rs)
* source compile gates in
  [tools/nuisc/tests/memory_compile.rs](../../tools/nuisc/tests/memory_compile.rs)
* lowering ordering checks in
  [tools/nuisc/src/lowering/tests_async_runtime.rs](../../tools/nuisc/src/lowering/tests_async_runtime.rs)
