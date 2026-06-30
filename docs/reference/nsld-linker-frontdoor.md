# Nsld Linker Frontdoor

`Nsld` is the Nuis linker toolchain member introduced on the `alpha-0.6.0`
line.

At this stage, `Nsld` is intentionally a frontdoor over the existing linker
contract logic in `nuisc::linker`. It does not yet claim to be the final
self-owned object linker. Its job is to give linker work a stable tool
boundary before the implementation is split out further.

## Current Role

`Nsld` currently owns:

* link-plan inspection from `nuis.build.manifest.toml`
* heterogeneous calculate plan visibility
* clock protocol visibility
* final-stage reporting
* the first independent CLI boundary for future linker work

`Nsld` does not yet own:

* final native object linking
* replacement of the host toolchain wrapper
* binary section assembly independent from `nuisc`
* stable linker script or relocation formats

## Commands

```sh
cargo run -p nsld -- status
cargo run -p nsld -- plan <nuis.build.manifest.toml>
cargo run -p nsld -- plan <artifact-output-dir> --json
```

When given an output directory, `Nsld` resolves
`nuis.build.manifest.toml` inside that directory.

## Boundary Rule

The compiler may know the shared structure of `nustar` registration,
artifact manifests, lifecycle metadata, and YIR contracts. It should not grow
hard-coded knowledge of each domain's private linker behavior.

`Nsld` should therefore evolve toward this shape:

```text
nuisc produces verified artifacts and manifests
  -> nsld consumes the link contract
  -> nsld freezes hetero clock/data order
  -> nsld assembles the Nuis-owned binary container
  -> host toolchain is used only as a wrapper when required
```

## Alpha-0.6.0 Meaning

For `alpha-0.6.0`, success means:

* linker truth has a named tool boundary
* existing `nuisc::linker` behavior remains reusable
* `Nsld` can inspect real build outputs
* clock protocol and hetero calculate metadata are visible from the linker
  frontdoor

This is the beginning of linker independence, not the end of linker work.
