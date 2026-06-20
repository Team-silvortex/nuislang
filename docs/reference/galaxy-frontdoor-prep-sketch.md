# `galaxy` Frontdoor Prep Sketch

This file captures the current safest way to begin a future GPU-side image
processing `Galaxy` package, now named `PixelMagic`, without prematurely coupling it to
temporary host scaffolding.

`PixelMagic` does not exist yet as a checked-in package. This file is a prep
contract for when we start. Current `galaxy_*` example names are retained as
historical scaffolds and should not be read as the final package name.

## Current Thesis

The current repository is finally strong enough to support a very small
host-to-shader closure:

```text
filesystem state
-> text report shaping
-> host I/O emission
-> shader profile
-> shader render
```

Today that means:

```text
cli_pgm_info_demo
-> cli_pgm_invert_demo
-> cli_pgm_threshold_demo
-> filesystem_io_report_recipe
-> shader_profile_demo
-> shader_render_profile_demo
```

The point of this route is not that `PixelMagic` should print reports forever. The
point is that the host-side read/report/emit closure is now separate enough
from shader execution that the first image-processing lane can grow without
reusing ad hoc debug glue as if it were architecture.

## Minimal `PixelMagic` Scope

The first checked-in `PixelMagic` surface should stay deliberately small.

Do not start with:

* many filters
* many texture formats
* many backends
* large asset pipelines
* host-side kitchen-sink orchestration

Do start with one narrow path:

1. host-side input description
2. one image-processing packet
3. one shader render/compute pass
4. one result emission shape

## First Useful Shape

The first useful `PixelMagic` lane should read like this:

```text
host input summary
-> PixelMagic packet
-> shader execution
-> host result summary
```

Concrete current anchors:

* tooling image-preprocess bridge:
  [tooling-image-preprocess-lane.md](/Users/Shared/chroot/dev/nuislang/docs/reference/tooling-image-preprocess-lane.md)
* host-side prep:
  [filesystem_io_report_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/std/filesystem_io_report_recipe.ns)
* checked-in CPU image companions:
  [cli_pgm_info_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_info_demo)
  [cli_pgm_invert_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_invert_demo)
  [cli_pgm_threshold_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/tooling/cli_pgm_threshold_demo)
* first checked-in `PixelMagic` seed scaffold:
  [pixelmagic_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_profile_demo)
* first checked-in `PixelMagic` packet consumer scaffold:
  [pixelmagic_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_packet_bridge_demo)
* first checked-in `PixelMagic` texture-resource handoff scaffold:
  [pixelmagic_texture_resource_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_texture_resource_demo)
* first checked-in `PixelMagic` project-shaped pipeline scaffold:
  [pixelmagic_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_pipeline_demo)
* first checked-in `PixelMagic` single-binary render scaffold:
  [pixelmagic_render_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_render_demo)
* shader profile floor:
  [shader_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_profile_demo)
* shader render floor:
  [shader_render_profile_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/shader_render_profile_demo)

## Recommended First Packet

The first `PixelMagic` packet should avoid pretending we already have a full image
runtime.

Prefer a tiny packet carrying only:

* source identity handle or logical asset handle
* width
* height
* operation kind
* one or two small numeric controls

Example mental model:

```text
PixelMagicPacket {
  source_handle,
  width,
  height,
  op_kind,
  amount,
  flags
}
```

This should be enough for a first lane such as:

* solid color fill
* grayscale
* threshold
* invert
* simple blur seed

The first lane does not need real file decode/encode. It only needs a stable
packet story and one end-to-end shader-backed execution route.

## Separation Rule

`PixelMagic` should preserve three separate concerns:

* host preparation:
  filesystem/path/text/io concerns
* domain packet shaping:
  image-processing intent and parameters
* shader execution:
  target, viewport, pipeline, packet-to-render lowering

Short rule:

* host lanes should not know shader-internal lowering details
* shader lanes should not absorb host reporting concerns
* `PixelMagic` should sit between them as a narrow packet/result contract

## First Checked-In Demo

When we add the first real `PixelMagic` demo, it should prefer this order:

1. read a host-side input description
2. emit a tiny host summary
3. build one `PixelMagic` packet
4. feed one shader profile/render lane
5. emit one host-side result summary

Recommended future names:

* `pixelmagic_packet_demo`
* `pixelmagic_render_demo`

Current checked-in closest bridge:

* [pixelmagic_packet_bridge_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_packet_bridge_demo)

Current checked-in closest texture-resource handoff anchor:

* [pixelmagic_texture_resource_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_texture_resource_demo)

Current checked-in closest project-shaped pipeline anchor:

* [pixelmagic_pipeline_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_pipeline_demo)

Current checked-in closest single-binary render anchor:

* [pixelmagic_render_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/domains/pixelmagic_render_demo)

Current next-step texture contract:

* [galaxy-texture-handoff-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/galaxy-texture-handoff-contract.md)

Recommended first reading order:

```text
cli_pgm_info_demo
-> cli_pgm_invert_demo
-> cli_pgm_threshold_demo
-> filesystem_io_report_recipe
-> pixelmagic_profile_demo
-> pixelmagic_texture_resource_demo
-> pixelmagic_pipeline_demo
-> shader_profile_demo
-> shader_render_profile_demo
-> pixelmagic_packet_demo
-> pixelmagic_render_demo
```

## Current Non-Goals

This prep sketch does not claim that we already have:

* a stable image asset ABI
* a stable shader texture resource ABI for `PixelMagic`
* a stable compute-only image lane
* a stable multi-backend graphics abstraction

Those are future steps.

For now, success means something much smaller:

* one narrow `PixelMagic` packet
* one shader-backed execution path
* one host-side report/result closure
