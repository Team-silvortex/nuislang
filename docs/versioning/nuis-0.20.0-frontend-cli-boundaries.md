# `nuis` `0.20.0` Frontend / CLI Boundary Notes

This file records the current practical boundary between:

* source frontend + `AST -> NIR` truth that is already defended by tests
* deeper `nuis` CLI / later-lowering routes that still reject some otherwise
  valid-looking source compositions

Use this file when a feature appears to "work in frontend tests" but still
fails once routed through `nuis dump-*`, full source compile, or project build
paths.

## Current Short Rule

As of early `0.20.*`:

* `if` / `match` / `await` / `?` composition is now real in frontend lowering
  across common expression positions
* that truth is currently best defended by `parse_nuis_module(...)` and
  `frontend::tests_try`
* deeper CLI / source-compile routes still contain older restrictions that are
  not the same thing as frontend invalidity

## What Is Now Frontend-Real

The following are currently covered by frontend/NIR tests:

* `?` inside call arguments
* `?` inside binary expressions
* `?` inside method receivers
* `?` inside struct field values
* multiple `?` points inside one expression
* `if` / `match` expressions nested inside those positions
* `await (if ...)?`
* `await (match ...)?`
* `(if ...)?` producing `Task<T>` before later `await`
* `(match ...)?` producing `Task<T>` before later `await`

Primary defended test surface:

* [tools/nuisc/src/frontend/tests_try.rs](../../tools/nuisc/src/frontend/tests_try.rs)

Readable single-file source anchor:

* [examples/ns/memory/hello_task_result_control_flow.ns](../../examples/ns/memory/hello_task_result_control_flow.ns)

Current promoted source compile-closure set:

* [hello_task_result_control_flow.ns](../../examples/ns/memory/hello_task_result_control_flow.ns)
* [hello_task_glm_status_path.ns](../../examples/ns/memory/hello_task_glm_status_path.ns)
* [hello_task_glm_lifecycle_path.ns](../../examples/ns/memory/hello_task_glm_lifecycle_path.ns)
* [hello_task_glm_value_path.ns](../../examples/ns/memory/hello_task_glm_value_path.ns)
* [hello_task_glm_compare.ns](../../examples/ns/memory/hello_task_glm_compare.ns)
* [hello_task_glm_observe.ns](../../examples/ns/memory/hello_task_glm_observe.ns)
* [hello_task_glm_boundary_compare.ns](../../examples/ns/memory/hello_task_glm_boundary_compare.ns)

## Current CLI / Deeper-Pipeline Boundary

The same source surface is still not yet uniformly accepted by every deeper
route, but a small checked-in task/GLM observation set now survives the deeper
source pipeline too.

Two practical examples already seen on the current mainline:

1. `async fn main(...)` scheduler boundary

The frontend can lower async helpers with parameters, but the current scheduler
still rejects async entry `main` functions that take parameters.

Practical rule:

* use parameterized async helpers
* keep `async fn main()` parameterless

2. branch-local task/runtime primitive boundary in deeper source compile

Some `nuis dump-ast` / `dump-nir` source routes still reject control-flow
branches that directly contain consuming task/runtime primitives such as
`spawn`, `join`, `join_result`, `timeout`, `lock`, or `unlock`.

Practical rule:

* hoist branch-local runtime-producing effects before the branch when possible
* let the branch choose between already-produced values

That is why the checked-in source example
[hello_task_result_control_flow.ns](../../examples/ns/memory/hello_task_result_control_flow.ns)
uses:

* `fetch(...)` first
* then `if` / `match` over `Result<Task<i64>, Error>`
* then `selected_task?`
* then `await`

instead of placing `spawn(...)`-originating effects directly inside each
control-flow branch.

Current note:

* this promoted source set is now defended in
  [tools/nuisc/src/frontend/tests_try.rs](../../tools/nuisc/src/frontend/tests_try.rs)
  for frontend/NIR truth, and in
  [tools/nuisc/tests/memory_compile.rs](../../tools/nuisc/tests/memory_compile.rs)
  for source compile-closure truth
* the remaining boundary is broader than this promoted source set

## Current Verification Strategy

For `0.20.*`, treat the following layers separately:

1. frontend/NIR reality

Use:

* `cargo test -p nuisc frontend::tests_try -- --nocapture`
* targeted `parse_nuis_module(...)` tests for checked-in `.ns` examples

2. source compile / CLI / deeper lowering reality

Use:

* `cargo run -p nuis -- dump-ast ...`
* `cargo run -p nuis -- dump-nir ...`
* `nuisc::pipeline::compile_source_path(...)`
* project compile tests under `tools/nuisc/tests`

If layer 1 passes and layer 2 fails, classify that as a pipeline-boundary gap,
not automatically as a frontend-language gap.

## `0.20.*` Follow-Up Work

The next cleanup pass should focus on:

* lifting more branch-local runtime restrictions out of later source compile
  routes
* aligning `nuis dump-*` behavior with frontend-proven control-flow truth
* turning frontend-only example anchors into full compile-closure examples when
  the deeper route is ready

## Short Decision Rule

When deciding where to place a new regression:

* if the question is "can source syntax + frontend lowering express this?",
  add a frontend/NIR test
* if the question is "can the current checked-in compile chain carry this all
  the way through?", add a compile/project test
* do not treat those two questions as interchangeable during `0.20.*`
