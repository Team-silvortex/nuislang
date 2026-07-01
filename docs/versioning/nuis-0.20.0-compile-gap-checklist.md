# `nuis` `0.20.0` Compile-Gap Checklist

This file is the short operational checklist for the early `0.20.*` line.

It tracks places where:

* source frontend / `AST -> NIR` truth is already real
* but the deeper compile chain, CLI route, or project route is not yet fully
  aligned

Read this together with:

* [nuis-0.20.0-frontend-cli-boundaries.md](nuis-0.20.0-frontend-cli-boundaries.md)
* [nuis-0.20.0-branch-runtime-lowering-matrix.md](nuis-0.20.0-branch-runtime-lowering-matrix.md)
* [nuis-0.20.x-to-alpha-bootstrap-roadmap.md](nuis-0.20.x-to-alpha-bootstrap-roadmap.md)

## Current Short Rule

For `0.20.*`, do not collapse these into one question:

* `can the frontend express and lower this honestly?`
* `can the whole current compile chain carry it all the way through?`

This checklist exists for the second question.

## Gaps To Close

### 1. Async Entry Boundary

Status:

* frontend async helpers with parameters are real
* current async entry `main` still must remain parameterless

Current rule:

* parameterized async logic goes into helpers
* `async fn main()` stays parameterless

Done when:

* either the scheduler/entry contract explicitly supports parameterized async
  entry
* or the restriction is documented as intentional and stable enough to stop
  appearing as an accidental surprise

### 2. Branch-Local Runtime Effect Boundary

Status:

* frontend control-flow + `?` + `await` composition is real
* deeper source compile / CLI routes still reject some branch-local consuming
  task/runtime primitives

Known examples:

* direct `spawn(...)`-originating task/runtime effects inside some lowered
  `if` / `match` branches
* related consuming primitives such as `join`, `join_result`, `timeout`,
  `lock`, or `unlock`

Current rule:

* hoist effectful runtime production before the branch when possible
* let the branch choose among already-produced values
* same-callee `spawn(...)` / `thread_spawn(...)` branches that differ only by
  argument values now have a checked-in `select inputs -> one runtime effect`
  lowering path, alongside the corresponding `join` / `join_result` /
  `thread_join` / `thread_join_result` / `timeout` / `cancel` /
  `mutex_lock` helper routes summarized in
  [nuis-0.20.0-branch-runtime-lowering-matrix.md](nuis-0.20.0-branch-runtime-lowering-matrix.md)

Done when:

* `nuis dump-*` / `compile_source_path(...)` accept the same honest branch
  shapes already proven in frontend/NIR tests
* or the restriction is enforced as a deliberate contract with explicit docs
  and companion examples

### 3. Example Compile-Closure Promotion

Status:

* some new `.ns` examples are currently strongest as frontend/NIR anchors
* a small checked-in task/GLM source anchor set has now been promoted into
  real source compile-closure regressions

Promoted source anchors:

* [hello_task_result_control_flow.ns](../../examples/ns/memory/hello_task_result_control_flow.ns)
* [hello_task_glm_status_path.ns](../../examples/ns/memory/hello_task_glm_status_path.ns)
* [hello_task_glm_lifecycle_path.ns](../../examples/ns/memory/hello_task_glm_lifecycle_path.ns)
* [hello_task_glm_value_path.ns](../../examples/ns/memory/hello_task_glm_value_path.ns)
* [hello_task_glm_compare.ns](../../examples/ns/memory/hello_task_glm_compare.ns)
* [hello_task_glm_observe.ns](../../examples/ns/memory/hello_task_glm_observe.ns)
* [hello_task_glm_boundary_compare.ns](../../examples/ns/memory/hello_task_glm_boundary_compare.ns)

Current protection:

* [tools/nuisc/src/frontend/tests_try.rs](../../tools/nuisc/src/frontend/tests_try.rs)
* [tools/nuisc/tests/memory_compile.rs](../../tools/nuisc/tests/memory_compile.rs)
* [tools/nuisc/tests/tooling_compile.rs](../../tools/nuisc/tests/tooling_compile.rs)

Current note:

* this source set now survives `compile_source_path(...)`
* the checked-in regressions also assert key lowered task/runtime observer
  shapes in `dump-yir`-equivalent coverage
* the first promotion in this set required the zero-field `cpu.struct`
  aggregate fix for unit enum payloads such as `Error.InvalidInput`
* the repository now also has a checked-in project-form native artifact
  closure anchor:
  [native_artifact_closure_demo](../../examples/projects/tooling/native_artifact_closure_demo)
  together with an AOT compile/package/launch smoke in
  [tools/nuisc/src/lib.rs](../../tools/nuisc/src/lib.rs)

Done when:

* more source examples can move from “frontend/NIR-true anchor” into “full
  compile closure anchor” without needing shape concessions that hide the
  feature being demonstrated

### 4. Gate Separation Discipline

Status:

* frontend tests now cover more real composition truth than some deeper routes
* this is good information, but only if the repo keeps the layers explicit

Current rule:

* frontend/NIR truth belongs in frontend tests
* compile-closure truth belongs in `compile_source_path(...)`, `dump-*`, and
  project compile tests

Done when:

* new regressions are consistently placed in the correct layer
* release/readiness docs stop implying that one passing layer automatically
  proves the other

## Current Fast Checks

Use these first while closing the gaps:

* `cargo test -p nuisc frontend::tests_try -- --nocapture`
* `cargo test -p nuisc frontend::tests_try::parses_memory_task_result_control_flow_example_into_nir -- --nocapture`
* `cargo test -p nuisc lowers_dynamic_ -- --nocapture`

Use these second when checking deeper alignment:

* `cargo run -p nuis -- dump-ast <source.ns>`
* `cargo run -p nuis -- dump-nir <source.ns>`
* `nuisc::pipeline::compile_source_path(...)`
* project compile tests under `tools/nuisc/tests`

## Exit Rule

This checklist is no longer active when most remaining compile-chain failures in
the mainline are:

* ordinary depth/coverage work
* or clearly documented intentional policy

and not:

* “frontend already says yes, but deeper current routes still say no for old
  reasons we have not cleaned up yet”
