# Memory `.ns` Examples

This folder contains ownership/lifetime-oriented front-end examples:

* `hello_glm.ns`
* `hello_borrow_end.ns`
* `hello_task_glm_scalar_payload.ns`
* `hello_task_glm_struct_payload.ns`
* `hello_task_glm_nested_struct_payload.ns`
* `hello_task_glm_text_payload.ns`
* `hello_task_glm_nested_text_struct_payload.ns`
* `hello_task_glm_origin.ns`
* `hello_task_glm_status_path.ns`
* `hello_task_glm_value_path.ns`
* `hello_task_glm_lifecycle_path.ns`
* `hello_task_glm_lifecycle.ns`
* `hello_task_glm_boundary_compare.ns`
* `hello_task_glm_lifecycle_compare.ns`
* `hello_task_glm_observe.ns`
* `hello_task_glm_compare.ns`
* `hello_task_glm_join_nonconsuming_probe.ns`

Current note:

* this folder is intentionally small right now because the ownership/lifetime path is still being hardened through `YIR`/verifier work rather than expanded into a broad source-level surface
* the current verifier-facing rule set is documented in
  [docs/reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

Useful commands:

```bash
cargo run -p nuis -- dump-yir examples/ns/memory/hello_glm.ns
cargo run -p nuis -- dump-nir examples/ns/memory/hello_borrow_end.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_scalar_payload.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_struct_payload.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_nested_struct_payload.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_text_payload.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_nested_text_struct_payload.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_origin.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_status_path.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_value_path.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_lifecycle_path.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_lifecycle.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_boundary_compare.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_lifecycle_compare.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_observe.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_compare.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_join_nonconsuming_probe.ns
```

Recommended task-reading note:

* [hello_task_glm_origin.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_origin.ns)
  is the smallest direct `spawn -> join` payload path
* [hello_task_glm_status_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_status_path.ns)
  is the narrowest current `join_result -> task_completed/task_timed_out/task_cancelled`
  path and is the closest single-file memory companion to
  [task_status_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_status_recipe.ns)
* [hello_task_glm_value_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_value_path.ns)
  is the narrowest current `spawn -> join_result -> task_completed -> task_value`
  path and is the closest single-file memory companion to
  [task_value_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_value_recipe.ns)
* [hello_task_glm_lifecycle_path.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_lifecycle_path.ns)
  is the narrowest current `timeout/cancel -> join_result -> task_timed_out/task_cancelled`
  path and is the closest single-file memory companion to
  [task_lifecycle_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_lifecycle_recipe.ns)
* [hello_task_glm_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_compare.ns)
  is the clearest current direct-vs-observed comparison companion to
  [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
* [hello_task_glm_boundary_compare.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_boundary_compare.ns)
  is the wider sibling companion for
  [task_compare_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/task_compare_recipe.ns)
  when you also want to see timeout/cancel lifecycle contrast in the same file
* [hello_task_glm_observe.ns](/Users/Shared/chroot/dev/nuislang/examples/ns/memory/hello_task_glm_observe.ns)
  widens that same path with timeout/observation shaping
