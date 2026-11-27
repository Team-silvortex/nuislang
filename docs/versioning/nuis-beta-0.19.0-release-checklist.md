# Nuis beta-0.19 Validation Checklist

The [snapshot](nuis-beta-0.19.0-snapshot.md) identifies the commit. From the
repository root, run the gates in the [Build workflow](../../.github/workflows/build.yml):

```sh
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo fetch --locked
python3 scripts/check-workspace-architecture.py
bash scripts/check-host-absolute-paths.sh
cargo build --workspace --locked
```

Keep the README and documentation entrypoints at or below 300 lines. Link to
the proof catalog rather than appending new lowering receipts. All five entry
documents must link to the current minor's snapshot and checklist.

For changes to task selection or ns-nova callbacks, run the focused checks:

```sh
cargo test --locked -p nuis --bin nuis dev_tensor -- --test-threads=1
cargo test --locked -p nuisc --test ns_nova_application_session -- --test-threads=1
```

The Ubuntu workspace build does not test the macOS render provider or prove
native/reference parity. The [mainline map](../current-mainline-map.md)
documents the separate device test and the current native admission boundary.
