# Type `.ns` Examples

This folder contains type- and aggregate-oriented front-end examples:

* `hello_struct.ns`
* `hello_ref_struct.ns`

Reading guidance:

* `hello_struct.ns`
  named struct declaration plus field use
* `hello_ref_struct.ns`
  struct fields carrying `ref` values and ownership-sensitive use

Useful commands:

```bash
cargo run -p nuis -- dump-nir examples/ns/types/hello_struct.ns
cargo run -p nuis -- dump-yir examples/ns/types/hello_ref_struct.ns
```
