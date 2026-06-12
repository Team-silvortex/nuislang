# Nuis 0.18.0 Loop Memory-Read Contract Sketch

This file captures the smallest realistic path for extending counted/chained
`while` lowering with memory-aware read support.

It intentionally follows the narrower "latter" direction:

* do not add arbitrary carry expressions first
* do not attempt full general iterative backedge lowering first
* instead add a small, explicit encoding for loop-safe memory reads

The goal is:

`let counted/chained loop carries consume a constrained class of read-only address observations before general loop-body memory lowering exists`

## Why This Exists

Current `while` support is split across layers:

* verifier semantics already understand conservative post-loop borrow/move state
* guarded `while` lowering can already carry small read-only address payloads in
  terminal guarded forms like `return load_value(head);`
* counted/chained loop lowering still only accepts carry updates expressible as
  linear source tags such as:
  * `add_current`
  * `add_carry0`
  * `mul_prev_carry0`

That last point is the real blocker.

The current loop node contracts cannot encode:

* `let acc = acc + load_value(head);`
* `let acc = acc + load_at(buffer, 0);`

even when:

* the read is read-only
* the pointer/buffer is loop-invariant
* the loop shape is otherwise already recognized as counted/chained

## Narrow Proposal

Add a constrained family of loop carry sources for read-only memory access.

### First-class source kinds

Candidate additions:

* `add_read_value_fixed`
* `add_read_at_fixed`

Optionally later:

* `mul_read_value_fixed`
* `mul_read_at_fixed`

The word `fixed` matters:

* the address input must be loop-invariant
* the buffer index for `load_at` must be loop-invariant
* no borrow creation/destruction is introduced inside the loop node itself

This is not "general expression in carry source".

It is only:

* loop-safe load from a stable structural pointer
* loop-safe load from a stable buffer slot

## Required Lowering Contract

Today loop-chain nodes encode carry updates using short source tags only.

To support fixed memory reads, the contract would need one extra payload form.

Candidate shape:

* existing:
  * `<carry_init> <carry_kind>`
* extended:
  * `<carry_init> <carry_kind> [<carry_read_arg0> <carry_read_arg1> ...]`

Concrete examples:

### Structural read

Source:

```nuis
let acc: i64 = acc + load_value(head);
```

Possible encoded args:

```text
acc_init add_read_value_fixed head_ptr
```

### Buffer slot read

Source:

```nuis
let acc: i64 = acc + load_at(buffer, 0);
```

Possible encoded args:

```text
acc_init add_read_at_fixed buffer_ptr slot_zero
```

## Invariants

The first version should only accept reads when all of the following hold:

* the read expression is directly in a recognized carry update
* the base pointer/buffer is loop-invariant
* the index, if present, is loop-invariant
* the read is read-only and already accepted by verifier authority rules
* no `borrow(...)`, `borrow_end(...)`, `store_*`, `free(...)`, or `move(...)`
  occur in the carry source

Short rule:

`fixed readable address observations may feed a carry; loop-local ownership effects still may not`

Current implementation note:

* verifier now has an explicit fixed-readable-source naming layer for
  `load_value(...)` and `load_at(...)`
* prepare/lowering remain responsible for proving loop-invariance and deciding
  whether the source can enter a counted/chained loop contract

## What This Does Not Solve

This proposal does not yet solve:

* loop-local `borrow(head)` followed by `load_value(head_ref)`
* loop-local `load_next(head_ref)` traversal carries
* owner mutation/free inside counted/chained loop bodies
* arbitrary expression trees inside carry updates
* dynamic buffer index reads that depend on `current` or previous carries
* write-bearing memory effects in loop nodes

Those remain future work.

## Why This Is Better Than Arbitrary Expr Carry First

Benefits:

* much smaller node-contract change
* easier verifier/lowering alignment
* preserves the current loop-node model as "structured summary", not mini IR
* gives `0.18.*` a real forward step for memory-aware loop lowering

Tradeoff:

* the extension is intentionally narrow and will not magically support general
  backedge memory loops

That tradeoff is good for this phase.

## Likely Implementation Order

1. extend loop carry source/rendering enums
2. teach `parse_loop_carry_update(...)` to recognize:
   * `acc = acc + load_value(fixed_ptr)`
   * `acc = acc + load_at(fixed_buffer, fixed_index)`
3. extend loop node arg encoding with fixed-read payloads
4. update handwritten/runtime/YIR consumers to understand the new kinds
5. add loop-flow and loop-basic regression tests
6. only then consider branching/conditional variants using the same fixed-read
   source kinds

## Suggested First Tests

Positive:

* counted loop with invariant `head: ref Node`
  * `let acc: i64 = acc + load_value(head);`
* counted loop with invariant `buffer: ref Buffer`
  * `let acc: i64 = acc + load_at(buffer, 0);`

Negative:

* carry source with `load_value(borrow(head))`
* carry source with `load_next(head_ref)`
* carry source with `load_at(buffer, current)`
* carry source with `store_at(...)`

## Recommendation

For the `0.18.*` line, this should be treated as the next realistic lowering
milestone:

`before general memory backedge loops, support fixed read-only address observations as explicit loop carry source kinds`
