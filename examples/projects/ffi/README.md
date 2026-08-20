# FFI Projects

This directory contains project-form host-boundary examples whose project
metadata is part of the contract under test.

Start with [owned_return_object_demo](owned_return_object_demo). It asks the
registered `official.cffi` Nustar for one opaque `ref FfiObject` return,
serializes its exact static size, read policy, and destructor authority into
the project host-FFI index, then builds and runs a native binary that returns
zero only after exact cleanup.

```bash
cargo run -p nuis -- build \
  examples/projects/ffi/owned_return_object_demo \
  "$TMPDIR/nuis_owned_return_object_project"
cargo run -p nuis -- run-artifact \
  "$TMPDIR/nuis_owned_return_object_project"
```

This is not a raw-pointer example. Generic object layouts, writes, Buffer
fallback, helper transfer, loops, tasks, and async escape remain closed.
