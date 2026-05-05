# Core `.ns` Examples

This folder contains the smallest front-end language examples:

* `hello_world.ns`
* `hello_int.ns`
* `hello_expr.ns`
* `hello_let_expr.ns`
* `hello_let_int.ns`
* `hello_let_text.ns`
* `hello_if.ns`
* `hello_call.ns`
* `hello_method.ns`

Recommended reading order:

* `hello_world.ns`
  smallest `mod cpu Main` entry
* `hello_expr.ns`
  arithmetic expression lowering
* `hello_let_expr.ns`
  named intermediate values
* `hello_if.ns`
  current conditional shape
* `hello_call.ns`
  simple function-call lowering

Useful commands:

```bash
cargo run -p nuis -- dump-ast examples/ns/core/hello_world.ns
cargo run -p nuis -- dump-nir examples/ns/core/hello_if.ns
cargo run -p nuis -- dump-yir examples/ns/core/hello_call.ns
```
