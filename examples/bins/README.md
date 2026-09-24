# Rebuilding Example Bundles

This directory no longer ships generated binaries, host shims, manifests or IR
dumps. The old window/kernel bundles were retired in the 2026-09-24 cleanup,
recorded in `beta-0.15.0`; their source projects and regression coverage remain.

Use the current compiler and an ignored output directory:

```bash
cargo run -p nuis -- build examples/projects/kernel_tensor_demo target/example-builds/kernel_tensor_demo
cargo run -p nuis -- build examples/projects/tooling/native_artifact_closure_demo target/example-builds/native_artifact_closure_demo
```

These commands run from the repository root and regenerate manifests alongside
their artifacts. Follow the [native artifact workflow](../../docs/reference/nuis-native-artifact-workflow.md)
for verification, Nsld handoff and execution rather than trusting a stale host
binary. Build success alone is not device-execution evidence.

For the current application route, start with
[ns_nova_image_showcase](../projects/domains/ns_nova_image_showcase).
The older [window_controls_demo](../projects/window_controls_demo) source is a
repair/probe route, not a prebuilt application guarantee.

Only this guide is versioned here. Local outputs are ignored; never force-add
generated bundles. Intentional binary compatibility fixtures belong beside the
tests that consume them. See the [cleanup policy](../../docs/repo-cleanup-candidates.md).
