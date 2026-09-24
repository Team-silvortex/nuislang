# Examples Freshness Audit

This is the current routing and retention policy. Removed alpha-era cleanup
wishlists remain in Git history, not as a second source of current truth.

## Current Routes

Follow the [mainline map](current-mainline-map.md) and the
[beta-0.15 checkpoint](versioning/nuis-beta-0.15.0-snapshot.md). A route's age or
name does not prove native execution, device completion or cross-platform support.

| Role | Source Route | Evidence Boundary |
| --- | --- | --- |
| Application frontdoor | [ns_nova_image_showcase](../examples/projects/domains/ns_nova_image_showcase) | Persistent image/window sessions; distinguish registered embedded-YIR providers from the bounded full-native scalar profile. |
| Basic CLI frontdoor | [filesystem_io_report_demo](../examples/projects/tooling/filesystem_io_report_demo), [stdin_runtime_demo](../examples/projects/tooling/stdin_runtime_demo), [cli_report_file_demo](../examples/projects/tooling/cli_report_file_demo) | Native host I/O and output-file evidence, not arbitrary-size or cross-platform certification. |
| Bounded CLI companions | [cli_wc_demo](../examples/projects/tooling/cli_wc_demo), [cli_pgm_invert_demo](../examples/projects/tooling/cli_pgm_invert_demo) | Bounded reads and small image fixtures; retain reference/performance harnesses. |
| Compiler/linker companions | [bootstrap_structural_projection_candidate](../examples/projects/tooling/bootstrap_structural_projection_candidate), [native_artifact_closure_demo](../examples/projects/tooling/native_artifact_closure_demo) | Exact candidate/finalizer acceptance boundaries, not full self-hosting. |
| Kernel source anchor | [kernel_tensor_demo](../examples/projects/kernel_tensor_demo) | Rebuild with the current toolchain; do not use the retired prebuilt binary as evidence. |

PixelMagic and WitSage host composition examples are not device execution by
name alone. Hardware-specific claims require the corresponding runtime evidence.

## Retention Rules

* Keep old-but-active parser, lowering, ownership, ABI, packaging and runtime
  regressions. Bug age is not a reason to discard protection.
* Keep negative fixtures that pin a current rejection contract.
* Keep source mirrors when explicit tests or focused documentation still use them.
* Keep handwritten YIR probes as verifier/domain coverage, separate from generated
  IR dumps.
* Remove a module only after checking Cargo targets, Rust module/path/include
  registration, macro registration, Nuis imports/manifests and runtime assets.
* Keep minor-line history in [versioning](versioning/README.md), not in misleading
  current inventories.

## Generated Outputs

The 2026-09-24 cleanup retires both checked-in bundles from `examples/bins/`.
The [rebuild guide](../examples/bins/README.md) replaces binary download links;
source examples and native artifact tests remain.

New examples build into `target/example-builds/` or a test-owned temporary
directory. Per-project `.nuis/cache` and tool-local `target` output are
regeneratable. Do not delete other project state merely because it lives inside
`.nuis`. Follow the [repository cleanup policy](repo-cleanup-candidates.md).
