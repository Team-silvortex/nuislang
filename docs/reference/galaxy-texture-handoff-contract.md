# `galaxy` Texture Handoff Contract

This file captures the current minimum contract for handing a preprocessed
image description into the future shader-facing texture/resource lane of the
`PixelMagic` image-processing `Galaxy`.

It is not a claim that the full `PixelMagic` texture ABI already exists.

It is the current smallest contract we can explain honestly after the
checked-in CPU preprocess lane and the first `galaxy_*` packet bridge scaffold.

## Current Position

Today the repository already has two real halves:

```text
tooling image preprocess
-> PixelMagic packet bridge
```

Concrete anchors:

* [tooling-image-preprocess-lane.md](tooling-image-preprocess-lane.md)
* [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
* [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)
* [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
* [pixelmagic_render_demo](../../examples/projects/domains/pixelmagic_render_demo)

The next half is not “many more image filters”.

The next half is:

```text
preprocessed image description
-> texture/resource description
-> shader-facing bindings
-> sampling/render consumer
```

## Minimal Goal

The smallest believable next contract is:

`one host-side preprocessed image description can be lowered into one narrow texture resource shape that a shader lane can bind and sample`

That goal is smaller than:

* full decode/encode pipelines
* asset catalogs
* multi-format image packs
* compute/image graph scheduling
* backend-specific texture upload code

Short rule:

`bind one believable texture shape first`

## Separation Rule

Keep these four concerns separate:

* host preprocess description
  width/height/op/result summary from the tooling lane
* `PixelMagic` image packet
  image-processing intent and image identity
* texture/resource handoff
  format/shape/filter/addressability metadata
* shader consumer
  target, bind set, sample path, render/dispatch result

Short rule:

* tooling should not guess shader binding layout details
* shader should not absorb host-side file/report logic
* `PixelMagic` should carry the packet-to-resource boundary

## Minimal Handoff Shape

The first stable handoff should stay tiny.

Prefer a shape like:

```text
PixelMagicImageResource {
  source_handle,
  width,
  height,
  format_kind,
  texel_layout,
  sampler_filter,
  address_mode
}
```

Meaning:

* `source_handle`
  logical host-side identity, not a promise of final asset storage
* `width` / `height`
  explicit image shape
* `format_kind`
  first narrow format family such as grayscale8 or rgba8
* `texel_layout`
  enough layout truth for backend/resource lowering
* `sampler_filter`
  first sampling policy, not a full sampler feature matrix
* `address_mode`
  first edge-handling policy

## First Narrow Format Rule

Do not begin with many formats.

The current safest first shape is one of:

* grayscale8-like single-channel logical shape
* rgba8_unorm-like four-channel logical shape

For the current checked-in PGM lane, the most honest first bridge is:

* one grayscale-oriented logical image description
* one narrow sampler policy
* one shader-facing sample path

That matches the current CPU preprocess truth better than pretending we
already have a rich color/atlas/material asset pipeline.

## Shader-Facing Surface

The current YIR-side texture/resource floor already exists conceptually through:

* `shader.texture2d`
* `shader.sampler`
* `shader.texture_binding`
* `shader.sampler_binding`
* `shader.sample`
* `shader.sample_uv`

Current references:

* [yir-langref.md](yir-langref.md)
* [nuis-ir.md](../../docs/grammar/nuis-ir.md)

The important current contract is not that every one of these already has a
finished high-level source wrapper.

The important current contract is:

`the future PixelMagic texture handoff should lower into this existing YIR resource/sampling vocabulary rather than inventing a parallel image-only abstraction`

## First Consumer Shape

The first useful checked-in consumer should read like:

```text
PixelMagicImagePacket
-> PixelMagicImageResource
-> shader texture binding
-> shader sampler binding
-> sample_uv
-> host-visible result
```

Current checked-in closest source-level anchor:

* [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)

Current checked-in closest project-shaped pipeline anchor:

* [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)

That can still be a render-style path before a compute-style path.

It does not need to begin as a general image kernel system. Current
`galaxy_*` example names remain temporary scaffolds rather than the intended
package name.

## Current Non-Goals

This contract does not claim that we already have:

* source-level `texture2d` authoring stabilized in `nuis`
* finished backend texture upload ABI
* many sampler/address/filter combinations
* mipmaps
* array textures
* storage-image compute semantics

Those can come later.

The first success condition is much smaller:

* one narrow image-resource shape
* one shader-facing binding route
* one sampled consumer path

## Reading Order

If you only want the shortest current route, read:

1. [tooling-image-preprocess-lane.md](tooling-image-preprocess-lane.md)
2. [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
3. [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)
4. [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
5. [pixelmagic_render_demo](../../examples/projects/domains/pixelmagic_render_demo)
6. [galaxy-frontdoor-prep-sketch.md](galaxy-frontdoor-prep-sketch.md)
7. [galaxy-texture-handoff-contract.md](galaxy-texture-handoff-contract.md)
8. [yir-langref.md](yir-langref.md)
9. [nuis-ir.md](../../docs/grammar/nuis-ir.md)
