# Tooling Image Preprocess Lane

This file captures the current repository contract for the checked-in
CPU-hosted image-preprocess companions in the tooling lane.

It is not a claim that `nuis` already has a finished image library.

It is the current shortest explanation for why the checked-in `pgm` companions
exist and how they connect to later shader-facing work.

## Current Thesis

The repository now has a small but real file-backed image lane:

```text
input file
-> narrow image probe
-> file-to-file transform
-> mask-style prepass
-> later shader/domain packet
```

Today that means:

```text
cli_pgm_info_demo
-> cli_pgm_invert_demo
-> cli_pgm_threshold_demo
```

The point of this route is not that ASCII PGM is the future public image ABI.

The point is that the current tooling/runtime surface can already prove one
important thing without shell glue:

`nuis can read a real file, validate a tiny image-shaped payload, transform it, and emit a new file through the native artifact path`

## Current Scope

This lane is deliberately narrow.

Today it proves:

* host argv/file input survives native AOT launch
* file-backed bytes can reach a checked-in `Buffer`
* tiny image-shaped validation can stay inside current lowering limits
* dynamic text handles can be written back through host file output
* project-form companions can exercise this path under `build -> run-artifact`

Today it does not prove:

* a stable public image asset ABI
* general image decode/encode support
* large image memory behavior
* parameter-rich CPU image processing
* a finished shader texture/resource upload contract

Short rule:

`this lane proves closure, not completeness`

## Current Companion Roles

### `cli_pgm_info_demo`

Anchor:

* [cli_pgm_info_demo](../../examples/projects/tooling/cli_pgm_info_demo)

Current job:

* prove file-backed image input can be opened and read
* prove a tiny checked-in sample shape can be validated
* stay as the narrowest image-shaped runtime probe

### `cli_pgm_invert_demo`

Anchor:

* [cli_pgm_invert_demo](../../examples/projects/tooling/cli_pgm_invert_demo)

Current job:

* extend the probe into a real file-to-file transform
* prove dynamic text serialization and host file write can round-trip through
  the native binary path
* act as the current CPU-side transform floor before shader-backed image
  examples

### `cli_pgm_threshold_demo`

Anchor:

* [cli_pgm_threshold_demo](../../examples/projects/tooling/cli_pgm_threshold_demo)

Current job:

* extend the same lane into a mask/prepass-shaped output
* act as the current closest CPU-side stand-in for a later shader prepass
* keep the current branch/lowering surface honest by preferring a stable,
  narrow sample over pretending we already have a fully generic threshold lane

## Why This Lane Lives In `tooling`

These companions are intentionally in the tooling lane first.

That does not mean image processing belongs to tooling forever.

It means the earliest believable closure is currently:

* CLI/file input
* CPU validation/reshaping
* file output
* artifact launch survival

That closure belongs to the current host/tooling surface more than it belongs
to a future shader package name. The intended image-processing `Galaxy` name
for that later lane is now `PixelMagic`.

Short rule:

`tooling proves the host-facing closure first; shader later proves the hetero execution path`

## Current Authoring Rule

For now, image-preprocess companions should prefer:

```text
argv capture
-> file open/read
-> tiny sample validation
-> narrow transform or prepass shaping
-> file write
-> one exit code
```

Practical rule:

* keep `main()` terminal and lowering-friendly
* prefer explicit host/file steps over hidden helper cleverness
* avoid pretending current lowering supports richer control flow than it really
  does
* choose one stable sample and one clear output per companion

## Shader-Facing Bridge

This lane is the CPU-side half of a later host-to-shader route.

The intended future bridge is:

```text
tooling image preprocess
-> tiny image packet
-> shader/domain packet shaping
-> shader execution
-> host-side result/report
```

The immediate next contract is not “many more CPU filters”.

The immediate next contract is:

* one stable preprocessed image description
* one narrow packet handoff shape
* one shader-facing consumer route

Current related references:

* [galaxy-frontdoor-prep-sketch.md](galaxy-frontdoor-prep-sketch.md)
* [galaxy-texture-handoff-contract.md](galaxy-texture-handoff-contract.md)
* [std-shader-kernel-project-contract.md](std-shader-kernel-project-contract.md)
* [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
* [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)

## Reading Order

If you only need the shortest current route, read:

1. [examples/projects/tooling/README.md](../../examples/projects/tooling/README.md)
2. [cli_pgm_info_demo](../../examples/projects/tooling/cli_pgm_info_demo)
3. [cli_pgm_invert_demo](../../examples/projects/tooling/cli_pgm_invert_demo)
4. [cli_pgm_threshold_demo](../../examples/projects/tooling/cli_pgm_threshold_demo)
5. [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
6. [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
7. [galaxy-frontdoor-prep-sketch.md](galaxy-frontdoor-prep-sketch.md)

## Current Success Condition

For the current line, success means something small and honest:

* one checked-in image-shaped probe
* one checked-in transform
* one checked-in mask/prepass
* all of them survive the current native artifact workflow
* the route is explainable as a staging lane for later hetero/shader work
