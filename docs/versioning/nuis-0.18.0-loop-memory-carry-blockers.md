# Nuis 0.18.0 Loop Memory Carry Blockers

This file names the next blockers after the current fixed-readable carry-source
step.

It exists so future work can start from explicit compiler boundaries instead of
re-deriving them from tests and lowering failures.

The examples in this file intentionally use builtin memory helper names such as
`load_at(...)` and `load_next(...)` because the blocker discussion is framed at
the carry-recognition / lowering-contract layer.

For source-facing address spelling, see
[../reference/address-surface-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/address-surface-contract.md).

Current implementation truth:

* verifier now names `load_value(...)` / `load_at(...)` as fixed readable
  carry-source candidates
* prepare/lowering can admit those reads only when their inputs are
  loop-invariant
* counted/chained loop contracts still reject dynamic index reads, structural
  traversal carries, and write-bearing memory backedges

Short rule:

`the next work is no longer “add memory loops”; it is “remove specific blockers one by one”`

## Blocker 1: Loop-Variant Buffer Indices

Example:

```nuis
let acc: i64 = acc + load_at(buffer, current);
```

Why it is blocked:

* current loop-node contracts can encode fixed buffer reads, not value-dependent
  slot selection
* the read payload is no longer a stable observation of one memory location
* `current` or previous carries would have to become first-class backedge input
  operands for the memory-read part of the contract

What would have to change:

* loop contract shape must distinguish fixed slot reads from dynamic slot reads
* prepare/lowering must prove which loop values are legal dynamic read drivers
* downstream runtime/YIR consumers must agree on how dynamic read operands are
  sequenced and interpreted

Primary risk:

`once the read location depends on loop state, the contract stops being a simple stable-memory summary`

## Blocker 2: Structural Traversal Carries

Example:

```nuis
let next_ptr: ref Node = load_next(head_ref);
let acc: i64 = acc + load_value(next_ptr);
```

Why it is blocked:

* `load_next(...)` changes the observed address itself across iterations
* borrowed traversal aliases preserve borrowed authority rather than regaining
  owner authority
* the loop contract would need to summarize not just a read, but address-state
  evolution on the backedge
* current compiler classification now names this separately from fixed reads and
  dynamic buffer-index reads, but still does not admit it into loop contracts

What would have to change:

* carry contracts must be able to encode address-valued backedge state, not
  only scalar accumulation summaries
* verifier/prepare/lowering must agree on how borrowed-derived traversal
  pointers survive across iterations
* loop lowering must preserve traversal ordering constraints already protected
  by async/runtime tests

Primary risk:

`structural traversal is not just “another read”; it is an address-transition problem`

## Blocker 3: Loop-Local Borrow Activity

Example:

```nuis
let head_ref: ref Node = borrow(head);
let acc: i64 = acc + load_value(head_ref);
```

Why it is blocked:

* verifier already reasons conservatively about loop-local borrows
* counted/chained loop contracts do not currently model borrow creation/end as
  explicit backedge state
* allowing borrow-local carry reads without stronger structure could make loop
  summaries hide authority transitions that are currently explicit in NIR

What would have to change:

* either loop contracts remain read-only over already-established aliases only
* or borrow lifecycle itself must become an encoded loop fact

Primary risk:

`authority changes inside the loop can become invisible if summarized too aggressively`

## Blocker 4: Write-Bearing Memory Effects

Examples:

```nuis
store_at(buffer, 0, value);
store_value(head, value);
free(head);
```

Why it is blocked:

* current counted/chained loop nodes summarize read/control/carry structure, not
  mutation ordering
* writes and frees need effect ordering, alias safety, and often ownership
  closure across the backedge
* this is much closer to general iterative memory lowering than to the current
  fixed-read extension

What would have to change:

* loop nodes would need explicit effect-bearing memory subcontracts
* verifier and lowering would need a stronger model for mutation visibility
  across iterations
* ordering guarantees would need to match the standalone runtime ordering tests

Primary risk:

`once writes enter the contract, the loop node starts acting like a mini memory IR`

## Blocker 5: Arbitrary Carry Expressions

Examples:

```nuis
let acc: i64 = mix(load_at(buffer, current), value);
let acc: i64 = acc + load_value(load_next(head_ref));
```

Why it is blocked:

* current carry recognition intentionally looks for small structured shapes
* arbitrary expression support would collapse the current summary model into a
  general expression-graph transport problem
* the compiler would lose the clean distinction between “recognized loop family”
  and “general loop lowering not implemented”

What would have to change:

* either carry expressions become a nested IR fragment inside loop contracts
* or the compiler grows a more general iterative/backedge lowering path first

Primary risk:

`arbitrary expr carry first is the shortest path back to an unbounded design`

## Suggested Order After Fixed Reads

The current best order still looks like:

1. keep fixed readable sources narrow and stable
2. decide whether dynamic buffer indices or structural traversal is the next
   narrower extension
3. avoid write-bearing loop-node contracts until general iterative memory
   lowering is much closer

Recommendation:

`if 0.18.* continues this line, dynamic index reads are probably the next honest narrow blocker to tackle; traversal and writes are larger jumps`
