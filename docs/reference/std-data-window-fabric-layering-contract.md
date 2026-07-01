# `std` Data / Window / Fabric Layering Contract

This file captures the current layering contract for the checked-in `std`
data-plane lanes built around `window`, `pipe`, `fabric`, and `handle_table`.

It sits one level below
[std-mainline-layering-contract.md](std-mainline-layering-contract.md):
that file explains the global `std` rule of thumb, while this file explains
what the data/window/fabric lane currently means in repository practice.

## Current Lane Shape

The current data/window/fabric lane prefers this order:

```text
current YIR/verifier data surface
-> narrow data/runtime recipes
-> wider route or bundle recipe
-> source/project companions
```

For checked-in `std`, that currently means:

```text
window / pipe / fabric / handle_table
-> window_fabric
-> examples/ns/ffi mirrors and examples/projects companions
```

## Verifier Boundary First

This lane should be read against the current `YIR` and verifier contract first,
not against the older fabric design drafts.

Current implementation-truth references:

* [yir-langref.md](yir-langref.md)
* [docs/grammar/nuis-ir.md](../../docs/grammar/nuis-ir.md)

Current design-background reference:

* [docs/fabric-spec/README.md](../../docs/fabric-spec/README.md)

The practical rule today is:

* current `data.*` ops are the instruction-level implementation surface
* current `std` pure runtime recipes are the narrow checked-in source-level
  contract for readable fabric/data routes
* older `fabric-spec/DFIR.md` material is useful background, but it is not the
  fastest source of current legality or layering truth

That means the pure `std` data/window/fabric layer is not inventing a new data
model. It is the checked-in front door for the current verifier-owned surface.

## Pure Data / Runtime Layers

These are the current narrow checked-in data-plane routes.

* [window_runtime_recipe.ns](../../stdlib/std/window_runtime_recipe.ns)
* [pipe_runtime_recipe.ns](../../stdlib/std/pipe_runtime_recipe.ns)
* [fabric_runtime_recipe.ns](../../stdlib/std/fabric_runtime_recipe.ns)
* [handle_table_runtime_recipe.ns](../../stdlib/std/handle_table_runtime_recipe.ns)

Current role split:

* `window_runtime` is the narrow local window route:
  `alloc_buffer -> copy_window -> write_window -> freeze_window -> read_window`
* `pipe_runtime` is the narrow pipe roundtrip route:
  `bind_core -> handle_table -> marker -> output_pipe -> input_pipe`
* `fabric_runtime` is the narrow fabric roundtrip route expressed directly
  through the current `data.*` bridge
* `handle_table_runtime` is the narrow resource-indirection and boot-shape
  setup route

These are the narrowest readable contracts for the current checked-in
data-plane surfaces.

## Wider Composition Layer

The current wider composition layer for this lane is:

* [window_fabric_recipe.ns](../../stdlib/std/window_fabric_recipe.ns)

Its current role is to combine:

* local window packing
* fabric/pipe roundtrip
* one wider summary route that shows how these pieces stack in one practical
  lane

The current rule is the same as the global `std` rule:

* `window_fabric` should not be the first place the `window`, `pipe`, `fabric`,
  or `handle_table` surfaces become understandable if the narrower pure layers
  can reasonably exist on their own

## Current Data / Window / Fabric Cluster

```text
window
-> pipe
-> fabric
-> handle_table
-> window_fabric
```

Concrete sources:

* [window_runtime_recipe.ns](../../stdlib/std/window_runtime_recipe.ns)
* [pipe_runtime_recipe.ns](../../stdlib/std/pipe_runtime_recipe.ns)
* [fabric_runtime_recipe.ns](../../stdlib/std/fabric_runtime_recipe.ns)
* [handle_table_runtime_recipe.ns](../../stdlib/std/handle_table_runtime_recipe.ns)
* [window_fabric_recipe.ns](../../stdlib/std/window_fabric_recipe.ns)

This cluster should currently be read as:

* `window` explains local window materialization and stable readback
* `pipe` explains explicit route crossing
* `fabric` explains the narrowest checked-in roundtrip route
* `handle_table` explains resource-carrier and binding metadata shape
* `window_fabric` explains the first practical combined route

## Current Legality Direction

The current lane should be read with these verifier-facing constraints in mind:

* `data.input_pipe` must consume a matching `data.output_pipe`
* nested pipe formation is rejected
* marker and handle carriers are not valid window sources
* mutable/local window shaping is intentionally separated from bridge-safe
  stable views
* `handle_table` remains a resource-indirection surface, not a plain data
  payload carrier

Those are not just style preferences. They are part of the current repository
meaning of this lane.

## Companion Expectation

The current checked-in data/window/fabric lane is expected to have direct
mirrors in:

* `examples/ns/ffi` for the source-level facade view
* `examples/projects/filesystem` for the current project-form route

Examples:

* [hello_window_runtime_facades.ns](../../examples/ns/ffi/hello_window_runtime_facades.ns)
* [hello_pipe_runtime_facades.ns](../../examples/ns/ffi/hello_pipe_runtime_facades.ns)
* [hello_fabric_runtime_facades.ns](../../examples/ns/ffi/hello_fabric_runtime_facades.ns)
* [hello_handle_table_runtime_facades.ns](../../examples/ns/ffi/hello_handle_table_runtime_facades.ns)
* [window_runtime_demo](../../examples/projects/filesystem/window_runtime_demo)
* [pipe_runtime_demo](../../examples/projects/filesystem/pipe_runtime_demo)
* [fabric_runtime_demo](../../examples/projects/filesystem/fabric_runtime_demo)
* [handle_table_runtime_demo](../../examples/projects/filesystem/handle_table_runtime_demo)

## What This Contract Does Not Promise

This file does not promise that:

* the current `window -> pipe -> fabric -> handle_table` order is a frozen ABI
* the checked-in `std` lane already captures the full long-range Fabric design
* the current filesystem project bucket is the final long-term home for these
  companions
* every future `data.*` promotion will keep the same naming or bundling shape

It only captures the current repository truth about how the checked-in
data/window/fabric lanes are meant to stack today.

## Current Guidance

If you are extending this lane today:

* add the narrow data/runtime recipe first when the surface can stand on its own
* keep verifier-legality expectations explicit before widening the route
* only add the umbrella route after the `window`, `pipe`, `fabric`, or
  `handle_table` layer is already readable
* treat `docs/fabric-spec` as background unless the current reference docs and
  verifier behavior say otherwise

If you are reading this lane today:

* start with [yir-langref.md](yir-langref.md)
  if you need the current `data.*` semantics
* start with the pure `*_runtime_recipe.ns` files if you need the narrow
  checked-in source contract
* move to [window_fabric_recipe.ns](../../stdlib/std/window_fabric_recipe.ns)
  only after the pure layers are clear

## Related References

* [std-mainline-layering-contract.md](std-mainline-layering-contract.md)
* [yir-langref.md](yir-langref.md)
* [docs/grammar/nuis-ir.md](../../docs/grammar/nuis-ir.md)
* [docs/fabric-spec/README.md](../../docs/fabric-spec/README.md)
* [cpu-task-external-handle-contract.md](cpu-task-external-handle-contract.md)
* [cpu-task-external-handle-glm-sketch.md](cpu-task-external-handle-glm-sketch.md)
* [docs/current-mainline-map.md](../../docs/current-mainline-map.md)
