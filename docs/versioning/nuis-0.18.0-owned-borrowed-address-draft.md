# Nuis 0.18.0 Owned / Borrowed Address Draft

This file is a forward-looking design draft for a future split between:

* owned address values
* borrowed address aliases

It is not a checked-in parser commitment for `0.18.0`.

Its role is narrower:

* make the possible future semantic split concrete
* show what would really need to change
* help decide later whether the extra surface complexity is worth it

For the concrete internal-first implementation sequence, also see:

* [nuis-0.18.0-internal-address-class-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-internal-address-class-plan.md)

## Current Baseline

Today both of these source shapes use the same surface type:

```ns
let head: ref Node = alloc_node(10, null());
let head_ref: ref Node = borrow(head);
```

That means:

* `head` is ownership-carrying
* `head_ref` is borrow-alias-carrying
* but both still render as `ref Node`

The current distinction lives in verifier state, not in source type shape.

## Draft Goal

The future split would try to make one thing clearer:

`surface types should say whether a pointer-like value owns the addressable object or only borrows access to it`

## Draft Shape

This document uses neutral placeholder names:

* owner surface: `Address<T>`
* borrow surface: `Borrowed<T>`

These are placeholders, not final syntax.

The real final spelling could also be:

* `Ptr<T>` / `Ref<T>`
* `Address<T>` / `View<T>`
* `ref T` / `view T`

The important part is the semantic split, not the exact words.

## Draft Semantic Model

### 1. Owner address values

Owner values would carry:

* mutation authority
* structural write authority
* free authority
* move authority

Example intent:

```ns
let head: Address<Node> = alloc_node(10, null());
```

Allowed operations:

* `move(head)`
* `store_value(head, 77)`
* `store_next(head, next)`
* `free(head)`
* `borrow(head) -> Borrowed<Node>`

### 2. Borrowed address aliases

Borrowed values would carry:

* readable access
* no free authority
* no structural write authority
* no move authority

Example intent:

```ns
let head_ref: Borrowed<Node> = borrow(head);
```

Allowed operations:

* `load_value(head_ref)`
* `load_next(head_ref)`
* `is_null(head_ref)`
* `borrow_end(head_ref)`

Rejected operations:

* `move(head_ref)`
* `store_value(head_ref, 77)`
* `store_next(head_ref, next)`
* `free(head_ref)`

## Draft Operation Table

`alloc_node(...)`

* returns owner address

`alloc_buffer(...)`

* returns owner address

`borrow(owner)`

* returns borrowed address

`borrow_end(alias)`

* returns `Unit`

`load_value / load_next / load_at`

* accept both owner and borrowed addresses, if readable

`store_value / store_next / store_at / free`

* accept owner addresses only

## Why This Would Help

Main benefits:

* type signatures explain authority more directly
* error messages can say “expected owned address” or “got borrowed address”
* async/task rejection messages become clearer
* the model becomes easier to teach than “same type, different verifier state”

## Why This Is Still Expensive

This would not be a cosmetic rename.

It would require coordinated changes in at least these layers:

* parser surface
* `AstTypeRef` representation
* `NirTypeRef` representation
* type inference for `borrow(...)`
* compatibility rules in frontend type checking
* `nir_verify` borrow tracking and write checks
* rendering / diagnostics
* examples and docs
* async/task boundary checks
* project/readme regression anchors

## Smallest Viable Internal Transition

If nuis ever moves toward this split, the safest implementation order is:

1. keep source syntax unchanged for one phase
2. introduce an internal distinction in `NIR` type metadata between owner and
   borrowed address classes
3. upgrade verifier and diagnostics to use that distinction
4. add tests and docs for the clearer error messages
5. only then decide whether source syntax should also split

Short rule:

`split the semantics first, split the spelling later`

## Suggested Error Shape

Future diagnostics should ideally become more specific than today.

Instead of:

* `cannot move borrowed pointer`

The future target shape could be closer to:

* `move(...) expects owned address, found borrowed address alias`

Instead of:

* `store_next cannot write borrowed pointer ...`

The future target shape could be:

* `store_next(...) requires owned structural address, found borrowed address`

## Recommendation For 0.18.0

Do not implement the split yet.

For `0.18.0`, this draft should stay a design aid only.

The right immediate work is still:

* stabilize the current `ref`-based address family
* keep memory/task/GLM rules honest
* strengthen compile and lowering gates
* postpone semantic split until the pointer baseline is broader and more mature

## Trigger To Revisit This Draft

Revisit this split when one or more of these become pressing:

* current `ref` diagnostics are no longer clear enough
* more address families appear beyond `Node` and `Buffer`
* mutation-vs-read authority becomes a recurring user confusion point
* async/task ownership rules would benefit materially from surface-level
  authority distinction
