# `nuis` `alpha-0.4.*` Documentation Sync Inventory

This file records the current documentation synchronization point for the
`alpha-0.4.*` line.

It is intentionally practical: it says which docs are the current front doors,
which claims are implementation-backed, and which areas should still use
careful alpha wording.

## Short Rule

`alpha-0.4.*` documentation should describe one connected toolchain, not a bag
of feature islands.

The present-tense route is:

```text
nuis project / source
  -> frontend
  -> NIR
  -> YIR
  -> verify
  -> lower
  -> package / artifact
  -> inspect / doctor / run probe
```

When a doc cannot prove that a step runs, it should call the step a contract,
sidecar, reference execution path, or probe.

## Current Front Doors

Use these first:

* [../current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [nuis-alpha-0.4-system-inventory.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-system-inventory.md)
* [nuis-alpha-0.4-mainline-hardening-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
* [../reference/nuis-frontdoor-surface-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-frontdoor-surface-reference.md)
* [../reference/nuis-native-artifact-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-native-artifact-workflow.md)
* [../reference/nuis-binary-format-protocol.md](/Users/Shared/chroot/dev/nuislang/docs/reference/nuis-binary-format-protocol.md)
* [../repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)

Older `0.20.*`, `0.19.*`, and `0.18.*` files remain valuable predecessor
anchors, but they are no longer the default route into the repository.

## Current Implementation Truth

These claims are safe to use as current implementation truth with normal alpha
caution:

* `nuis` and `nuisc` form the main frontdoor/compiler split.
* Multi-file `nuis.toml` projects are the strongest examples layer.
* `NIR` and `YIR` are both real checked compiler surfaces.
* `YIR` verifier checks cover GLM-style ownership, data fabric, scheduler
  contracts, result-family states, CPU heap protocol, and lowering contracts.
* `nustar` manifests register domain families, default lanes, ABI targets, and
  clock metadata.
* The compiler should know contract shapes and registration structure, not
  hard-wire backend internals.
* AOT artifact emission now includes manifests, payload blobs, bridge/lowering
  indexes, domain unit sidecars, and artifact inspection commands.
* Host-YIR runtime probes can consume artifact YIR sidecars and execute
  registered domain mods for selected paths.
* `std`, `PixelMagic`, and `WitSage` are active official library/galaxy
  proving grounds.
* `ns-nova` is an official future GUI/rendering galaxy, but it should remain
  secondary until AOT, std, PixelMagic, and WitSage are firmer.

## Current Codebase Hygiene Snapshot

This sync follows a repository slimming pass.

Current non-test Rust source under `tools/nuisc/src` is about `106k` lines.
The largest non-test source files are now under `600` lines; the biggest
remaining files are near the threshold rather than far beyond it.

Practical rule:

* keep large implementation files split by responsibility before they cross
  the project policy threshold again
* prefer small facade modules that preserve existing public entrypoints
* do not split tests only for line count unless the test file is actively
  blocking maintenance

Related policy:

* [../repo-file-line-policy.md](/Users/Shared/chroot/dev/nuislang/docs/repo-file-line-policy.md)
* [../repo-cleanup-candidates.md](/Users/Shared/chroot/dev/nuislang/docs/repo-cleanup-candidates.md)

## Library And Galaxy Status

Current wording should distinguish these layers:

* `core`
  smallest stdlib semantic base and long-lived source contracts
* `std`
  practical systems layer for text, IO, filesystem, task/thread, network,
  errors, benchmark/reporting, and host-backed facades
* `PixelMagic`
  official image/resource galaxy and current shader-facing proving ground
* `WitSage`
  official classical ML galaxy and current kernel-facing proving ground
* `ns-nova`
  official future GUI/rendering galaxy; important, but not the next maturity
  bottleneck

Docs should not imply that these are final crate-style automatic imports yet.
They are checked-in source assets plus manifest-registered surfaces, with
selected auto-injectable library modules.

## Example Status

Use examples as proof routes, not as a flat catalog.

Preferred current routes:

* [../../examples/projects/window_controls_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/window_controls_demo)
* [../../examples/projects/kernel_tensor_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/kernel_tensor_demo)
* [../../examples/projects/tooling/native_artifact_closure_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/native_artifact_closure_demo)
* [../../examples/projects/domains/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/README.md)
* [../../examples/ns/README.md](/Users/Shared/chroot/dev/nuislang/examples/ns/README.md)
* [../../examples/yir/README.md](/Users/Shared/chroot/dev/nuislang/examples/yir/README.md)

Generated snapshots under `examples/bins` are useful rebuild artifacts, not the
canonical source of truth.

## Soft Areas

Use careful wording for:

* self-hosting
* final linker ownership
* final native container format stability
* final GLM/ownership treatment for raw pointers
* full GPU vendor backend maturity
* full NPU/ANE backend maturity
* real network service runtime beyond current contracts/probes
* ns-nova engine maturity
* complete source-level execution for every YIR capability

## Documentation Update Rule

When updating docs during `alpha-0.4.*`:

1. route readers through the current mainline map first
2. keep predecessor documents linked but demoted
3. say whether a surface is a contract, source asset, project example,
   sidecar, reference executor path, or true runtime path
4. update examples and stdlib docs together when a library surface changes
5. update this file when the repo-wide interpretation changes

