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
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_lifecycle.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_boundary_compare.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_lifecycle_compare.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_observe.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_compare.ns
cargo run -p nuis -- check examples/ns/memory/hello_task_glm_join_nonconsuming_probe.ns
```
