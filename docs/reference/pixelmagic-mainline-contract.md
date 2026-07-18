# `PixelMagic` Mainline Contract

This file is the shortest current contract for reading `PixelMagic` as a real
checked-in standard-library `Galaxy`, not just as a future package name.

## Current Position

Today `PixelMagic` already exists in two complementary forms:

* stdlib canonical source modules
* project-shaped domain companions

The important current rule is:

`stdlib defines the canonical chain; project demos prove the chain survives as one compiled domain route`

## Stdlib Chain

The current canonical source chain in
[stdlib/pixelmagic](../../stdlib/pixelmagic/README.md)
is:

```text
image packet
-> image resource
-> texture binding
-> sample intent
-> shader packet
-> shader consumer
-> project-shaped pipeline
```

Concrete current anchors:

1. [image_packet_recipe.ns](../../stdlib/pixelmagic/core/image_packet_recipe.ns)
2. [image_op_contract_recipe.ns](../../stdlib/pixelmagic/core/image_op_contract_recipe.ns)
3. [image_resource_recipe.ns](../../stdlib/pixelmagic/core/image_resource_recipe.ns)
4. [texture_binding_recipe.ns](../../stdlib/pixelmagic/core/texture_binding_recipe.ns)
5. [sampling_recipe.ns](../../stdlib/pixelmagic/core/sampling_recipe.ns)
6. [shader_packet_recipe.ns](../../stdlib/pixelmagic/core/shader_packet_recipe.ns)
7. [shader_consumer_recipe.ns](../../stdlib/pixelmagic/core/shader_consumer_recipe.ns)
8. [pixelmagic_pipeline_recipe.ns](../../stdlib/pixelmagic/core/pixelmagic_pipeline_recipe.ns)

## Domain Route

The current checked-in project route reads like:

```text
tooling preprocess
-> PixelMagic packet bridge
-> PixelMagic texture-resource handoff
-> PixelMagic project-shaped pipeline
-> PixelMagic render
```

Concrete current anchors:

1. [cli_pgm_info_demo](../../examples/projects/tooling/cli_pgm_info_demo)
2. [cli_pgm_invert_demo](../../examples/projects/tooling/cli_pgm_invert_demo)
3. [cli_pgm_threshold_demo](../../examples/projects/tooling/cli_pgm_threshold_demo)
4. [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
5. [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)
6. [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
7. [pixelmagic_render_demo](../../examples/projects/domains/pixelmagic_render_demo)

## Relationship Rule

The current intended relationship is:

* stdlib recipes should name the stable semantic chain
* project demos should pressure-test the same chain in one domain-shaped closure
* demos may still repeat logic that stdlib already models, but that duplication
  should shrink over time

Short rule:

`recipe first for contract truth; demo second for end-to-end lowering truth`

## Official Surface Registry

The current registry-facing `PixelMagic` surface ids are:

1. `contract.pixelmagic.image-resource-shaping.v1`
2. `contract.pixelmagic.texture-handoff.v1`
3. `contract.pixelmagic.shader-facing-image-prep.v1`
4. `surface.pixelmagic.shader.contracts.v1`
5. `surface.pixelmagic.shader.packet-bridge.v1`
6. `surface.pixelmagic.shader.render.v1`
7. `surface.pixelmagic.shader.texture.v1`
8. `surface.pixelmagic.shader.pipeline.v1`

The intended rule is:

* `contract.*` ids name semantic lowering/bridge commitments
* `surface.*` ids name checked-in shader-facing public units
* library module filenames may evolve, but these registry ids should remain the stable discovery vocabulary

## What Is Already Real

At the current repository stage, `PixelMagic` already has:

* an official stdlib package identity
* a canonical recipe chain in `stdlib/pixelmagic/core`
* an explicit shared image-op contract in stdlib form
* a checked-in project-shaped domain pipeline
* shader/data/cpu cooperation through the current packet/resource/render route
* a registered Metal provider runner that submits a real compute command buffer
  on macOS and records the Metal device plus output evidence

The current native sample deliberately proves the backend boundary with a
scalar image-byte-count operation. It is real Metal execution, but it is not
yet a full PGM pixel-buffer upload, filter dispatch, and readback pipeline.

## What Is Not Done Yet

`PixelMagic` does not yet claim:

* a stable public image asset ABI
* a finished source-level texture sampling surface
* a real import-based package workflow
* a finished public filter family API
* a backend-complete texture upload/runtime contract
* full image-buffer execution through the current Metal provider runner

## Reading Order

If you only want the shortest current reading route, use:

1. [stdlib/pixelmagic/README.md](../../stdlib/pixelmagic/README.md)
2. [stdlib/pixelmagic/core/README.md](../../stdlib/pixelmagic/core/README.md)
3. [pixelmagic_pipeline_recipe.ns](../../stdlib/pixelmagic/core/pixelmagic_pipeline_recipe.ns)
4. [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
5. [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)
6. [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
7. [pixelmagic_render_demo](../../examples/projects/domains/pixelmagic_render_demo)
