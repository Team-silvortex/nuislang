# NIR Optimization Contract

This file describes the current contract that conservative `NIR` optimization
passes are expected to follow.

It is intentionally narrower than a future optimizer design document. Right now
it defines what the repository treats as safe transformation territory for the
front-door compiler pipeline.

## Current Rule of Thumb

Optimization must not invent memory or runtime semantics.

Today that means:

* ownership/lifetime correctness remains owned by
  [nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)
* optimizations should prefer expression-local and function-local rewrites
* dead-code cleanup must stay away from effectful expressions unless there is an
  explicit contract allowing it

## Canonical Effect Classes

`nuis-semantics` now classifies `NirExpr` into eight coarse classes:

* `Pure`
  scalar literals, scalar binaries, pure struct literals, field projection from
  pure values, `is_null`, and similar value-only computation
* `LocalReadOnly`
  expression forms that read local ownership/lifetime state without mutating it,
  such as `borrow(...)`, `borrow_end(...)`, `load_value(...)`, and `load_at(...)`
* `HostReadOnly`
  expression forms that observe host/runtime-facing state without being modeled
  as mutating it in the current verifier contract, such as `cpu_tick_i64(...)`,
  `cpu_input_i64(...)`, `cpu.bind_core`, and current host-side
  target/pipeline descriptors
* `DomainReadOnly`
  expression forms that observe staged domain state without being modeled as
  mutating it in the current verifier contract, such as data/window adapters and
  current `data / shader / kernel` profile references
* `AsyncOpaque`
  expression forms whose scheduling or completion semantics are not yet trusted
  by front-door optimization, such as `await(...)`
* `CallOpaque`
  expression forms whose callee behavior is not modeled tightly enough yet for
  optimization, such as direct `call(...)` and `method_call(...)`
* `DomainOpaque`
  expression forms that cross into domain/unit instantiation boundaries without
  a stronger optimization contract yet, such as `instantiate`
* `Stateful`
  expression forms that allocate, consume ownership, write, free, emit to host
  interfaces, or otherwise have current side-effect weight, such as
  `move(...)`, `alloc_*`, `store_*`, `free(...)`, `cpu.extern_call_*`,
  task/present operations, and `data_profile_send_*`

Current implementation source:

* [model.rs](/Users/Shared/chroot/dev/nuislang/crates/nuis-semantics/src/model.rs)
  via `nir_expr_effect_class(...)`
* [host-read-bridge.md](/Users/Shared/chroot/dev/nuislang/docs/reference/host-read-bridge.md)
  for the narrower host-facing bridge names such as `clock_tick`,
  `host_main_lane`, and `worker_lane`

## What Current Passes Are Allowed To Do

The current optimizer path in
[optimize.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/optimize.rs)
is allowed to:

* fold constant integer arithmetic
* fold `is_null(null)` into `true`
* propagate literal bindings inside one function
* normalize `if true` / `if false`
* remove dead scalar bindings only when the bound expression is classified as
  `Pure`

## What Current Passes Must Not Assume

Current passes must not assume:

* that `LocalReadOnly`, `HostReadOnly`, or `DomainReadOnly` are removable
* that host/domain reads are interchangeable with pure computation
* that `AsyncOpaque`, `CallOpaque`, or `DomainOpaque` forms can be treated as if
  they were runtime reads
* that ownership-affecting expressions can be reordered or dropped
* that branch-local aliasing can be simplified without verifier-aligned rules

That is especially important for:

* `ref / borrow / move / free`
* host-backed CLI/runtime facades
* domain bridges such as `data`, `shader`, and `kernel`

## Current Boundary

This is still a staging optimizer contract.

It is strong enough for:

* early CLI-oriented CPU code
* conservative source cleanup
* future memory-aware optimization work

It is not yet a license for broad common-subexpression elimination, effect
reordering, alias-sensitive dead-store elimination, or cross-domain scheduling
optimization.
