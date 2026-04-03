# Invalid Examples

These examples are supposed to fail verification or front-end checks.

Subfolders:

* [ns/core](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/core/README.md)
  invalid front-end structure and unit-binding examples
* [ns/memory](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/README.md)
  invalid front-end ownership/lifetime examples
* [ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns)
  invalid front-end examples
* [projects](/Users/Shared/chroot/dev/nuislang/examples/invalid/projects)
  invalid multi-mod project examples
* [yir](/Users/Shared/chroot/dev/nuislang/examples/invalid/yir)
  invalid handwritten `YIR` examples

Recommended checks:

```bash
cargo run -p nuis -- check examples/invalid/ns/core/hello_bad_unit.ns
cargo run -p nuis -- check examples/invalid/ns/core/hello_nested_mod_invalid.ns
cargo run -p nuis -- check examples/invalid/projects/bad_links_missing_downlink
cargo run -p yir-run -- examples/invalid/yir/cpu_use_after_free_invalid.yir
```
