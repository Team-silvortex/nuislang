# Nuis 0.18.0 Internal Address Class Plan

This file turns the owned/borrowed address draft into a concrete internal
implementation plan.

The scope is intentionally limited:

* no parser change
* no checked-in source syntax split
* no `Ptr<T>` / `Address<T>` rename

The goal is only:

`make owner-vs-borrowed address authority visible inside compiler metadata and diagnostics before changing source syntax`

## Why This Plan Exists

Today the compiler already knows an important semantic fact:

* some `ref T` values are ownership-carrying
* some `ref T` values are borrow aliases

But that distinction is mostly carried in verifier state, not in `NIR` type
metadata.

That makes three things harder than they need to be:

* diagnostics
* future source-surface redesign
* targeted tests for authority-sensitive behavior

## Phase 1: Metadata Only

Target:

* add an internal address-class distinction to `NirTypeRef`-adjacent logic
* keep existing `ref T` rendering unchanged

Suggested shape:

* add a new enum in `crates/nuis-semantics/src/model.rs`

Candidate:

```rust
pub enum NirAddressClass {
    Owned,
    Borrowed,
}
```

Important constraint:

* do not immediately add this as a serialized/parser-facing source field unless
  we are sure it belongs permanently in the core type shape

Safer first step:

* expose helper classification APIs near `NirTypeRef`
* let expression/type inference produce address-class-aware answers

## Phase 2: Typed Expression Classification

Target:

* make expression typing distinguish authority class even when surface type
  stays `ref T`

Current critical split:

* `alloc_node(...)` -> owned address
* `alloc_buffer(...)` -> owned address
* `borrow(x)` -> borrowed address
* `move(x)` -> preserves address class of `x`
* `load_next(x)` -> currently returns `ref Node`, but authority should be
  decided intentionally

This phase needs one explicit policy decision:

### Resolved decision: `load_next(x)` inherits input class

The current internal rule is now:

* `load_next(owner)` -> owned
* `load_next(borrowed)` -> borrowed

This keeps structural ownership-link behavior for owner traversals while also
preventing a borrowed traversal path from silently regaining owner authority.

## Phase 3: Verifier Upgrade

Target:

* stop relying only on `borrow_bindings` string tracking for semantic clarity
* use address-class-aware checks where possible

Current hotspots:

* [tools/nuisc/src/nir_verify.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/nir_verify.rs)
* [tools/nuisc/src/nir_verify/task_result_facts.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/nir_verify/task_result_facts.rs)

First upgrades:

* `move(...)` error should know when the source is borrowed-address-class
* `store_value / store_next / store_at / free` checks should report whether the
  failing value lacked owner authority
* `borrow_end(...)` diagnostics should explicitly mention borrowed alias class

## Phase 4: Diagnostic Rewrite

Target:

* keep semantics unchanged
* make errors explain authority instead of only naming “borrowed pointer”

Current messages:

* `cannot move borrowed pointer`
* `store_next cannot write borrowed pointer ...`

Better target shape:

* `move(...) expects owned address, found borrowed address alias`
* `store_next(...) requires owned structural address, found borrowed address`
* `free(...) requires owned address, found borrowed address alias`

## Phase 5: Test Expansion

Target:

* test the new internal distinction without changing source syntax

Primary test families:

* [tools/nuisc/src/frontend/tests_types_async_window.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_types_async_window.rs)
* [tools/nuisc/src/nir_verify/tests.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/nir_verify/tests.rs)
* [tools/nuisc/tests/memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)

Needed additions:

* `borrow(...)` classified as borrowed address class
* `alloc_node / alloc_buffer` classified as owned address class
* `borrow_end(...)` consumes borrowed-alias state but does not create a new
  address value
* diagnostics mention owned vs borrowed authority

## Phase 6: Re-evaluate Source Syntax

Only after phases 1-5 are stable should we ask:

* does `ref T` remain good enough?
* should we rename it to `Address<T>` or `Ptr<T>`?
* should we split the source surface into owner and borrowed forms?

Short rule:

`do not change source spelling until internal authority semantics and diagnostics are already solid`

## Minimal Code Touch Order

The lowest-risk implementation order is:

1. `crates/nuis-semantics/src/model.rs`
   add internal address-class helpers/types
2. `tools/nuisc/src/frontend/types/nir.rs`
   add expression-level authority classification
3. `tools/nuisc/src/nir_verify/task_result_facts.rs`
   centralize borrowed-address detection behind richer helpers
4. `tools/nuisc/src/nir_verify.rs`
   upgrade authority-sensitive checks and diagnostics
5. tests
   first semantics tests, then compile-gate checks

## Out Of Scope For This Plan

This plan does not yet include:

* generalized pointer arithmetic
* arbitrary address casts
* cross-domain pointer transfer
* async-safe moved-owner semantics
* parser-visible `Address<T>` / `Borrowed<T>` syntax

## Recommendation

If we want the next concrete engineering step after the docs phase, this is the
right one:

* implement internal address-class helpers first
* upgrade diagnostics second
* delay any syntax split until later
