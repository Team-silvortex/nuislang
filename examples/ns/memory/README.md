# Memory `.ns` Examples

This folder contains ownership/lifetime-oriented front-end examples:

* `hello_glm.ns`
* `hello_borrow_end.ns`

Current note:

* this folder is intentionally small right now because the ownership/lifetime path is still being hardened through `YIR`/verifier work rather than expanded into a broad source-level surface
* the current verifier-facing rule set is documented in
  [docs/reference/nir-memory-model.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nir-memory-model.md)

Useful commands:

```bash
cargo run -p nuis -- dump-yir examples/ns/memory/hello_glm.ns
cargo run -p nuis -- dump-nir examples/ns/memory/hello_borrow_end.ns
```
