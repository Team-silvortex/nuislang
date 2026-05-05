# Memory `.ns` Examples

This folder contains ownership/lifetime-oriented front-end examples:

* `hello_glm.ns`

Current note:

* this folder is intentionally small right now because the ownership/lifetime path is still being hardened through `YIR`/verifier work rather than expanded into a broad source-level surface

Useful commands:

```bash
cargo run -p nuis -- dump-yir examples/ns/memory/hello_glm.ns
```
