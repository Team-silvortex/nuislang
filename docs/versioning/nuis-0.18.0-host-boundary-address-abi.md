# Nuis 0.18.0 Host Boundary Address ABI

This file records the current honest rule for pointer-like values at the host /
`extern` boundary.

It exists because two facts are both true at once:

* current nuis already has a real internal address core
* current `extern` lowering does not yet expose a stabilized pointer ABI

Short rule:

`inside nuis, ref is real; across extern, ref is not stabilized yet`

## Current Internal Truth

The current internal address surface is:

* `ref Node`
* `ref Buffer`
* `borrow(...)`
* `borrow_end(...)`
* `alloc_node / load_value / load_next / store_value / store_next`
* `alloc_buffer / load_at / store_at / buffer_len`

These are not parser-only decorations.

They already participate in:

* frontend type checking
* owned-vs-borrowed authority classification
* verifier move/write/free checks
* lowering of structural and buffer memory operations

Primary anchors:

* [nuis-0.18.0-address-pointer-mainline.md](nuis-0.18.0-address-pointer-mainline.md)
* [nir-memory-model.md](../../docs/reference/nir-memory-model.md)

## Current Host Boundary Truth

By contrast, ordinary `CpuExternCall` lowering is still value-shaped.

Today the important practical truth is:

* `extern "c"` calls lower through the generic CPU extern path
* that path still materializes an `extern_call_i64`-style operation
* there is not yet a stable checked pointer ABI for host parameters or returns

This means current nuis must not pretend that these are already equivalent:

* internal pointer/address values
* host-visible pointer ABI values

## Current Compile Rule

For `0.18.0`, the current compiler rule is intentionally conservative:

* `extern fn foo(x: ref T) -> ...` is rejected
* `extern fn foo(...) -> ref T` is rejected
* the same rule applies to `extern interface` methods

There is one intentionally narrow bridge on top of that rule:

* when an `extern` parameter is declared as plain `i64`
* and the call site passes `ref Buffer`
* the frontend currently accepts that as a host buffer-handle bridge

The preferred explicit spelling for this route is now:

* `host_buffer_handle(buffer_ref)`

This is not general pointer ABI support.

It is only the current explicit escape hatch for buffer-backed host read/write
surfaces such as stdin/file transport facades.

Current validation anchor:

* [validation.rs](../../tools/nuisc/src/frontend/validation.rs)

Short rule:

`extern surfaces are value-only for now, with one narrow ref-Buffer-to-i64 buffer-handle bridge`

## Why The Rule Exists

Without this gate, the language would imply a stronger contract than the
compiler/runtime actually provide.

That would blur three different layers:

* internal pointer semantics
* async/task transfer semantics
* host ABI / syscall / runtime transport semantics

Keeping them separate is what lets the current pointer work stay honest.

## Relationship To Async Boundaries

Async/task boundaries and host/extern boundaries are related, but not the same
problem.

Current async rule:

* `spawn(...)` still rejects `ref` inputs
* diagnostics now distinguish borrowed, traversal-derived borrowed, and owned
  pointer cases

Current host rule:

* ordinary `extern` declarations reject `ref` in the ABI surface entirely
* call sites may still bridge `ref Buffer` into an `i64` host slot for the
  narrow buffer-handle route

Short rule:

`async boundaries know more about pointer classes; host boundaries still forbid general pointer ABI and only allow a narrow buffer-handle bridge`

## What This Unblocks

This conservative split makes the next steps clearer:

1. keep strengthening internal pointer ownership and lowering rules
2. keep `extern` ABI honest and narrow
3. add dedicated host/runtime pointer surfaces only when they have a real ABI
   story

That is the right foundation for future work like:

* stdin/stdout runtime maturity
* syscall-backed host text / buffer routes
* network handle plus owned-buffer transport surfaces
* explicit host-owned or borrowed buffer recipes

## What Needs To Exist Before Pointer ABI Opens Up

Before `extern` can safely grow `ref` parameters or returns, we need at least:

* an explicit lowered representation for host-visible pointer arguments
* ownership-transfer rules for host calls
* borrowed-pointer rejection or stabilization at the ABI edge
* runtime/linker/AOT agreement on how those values are materialized
* compile gates that distinguish internal address operations from host pointer
  transport

Until then, the stable rule should remain:

`internal pointer core first, host pointer ABI later`
